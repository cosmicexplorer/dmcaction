/* Copyright 2022 Danny McClanahan */
/* SPDX-License-Identifier: AGPL-3.0-only */

//! ???

/* Turn all warnings into errors! */
/* #![deny(warnings)] */
/* Warn for missing docs in general, and hard require crate-level docs. */
/* #![warn(missing_docs)] */
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
/* Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be
 * copied and pasted across crates, but there doesn't appear to be a way to include inner attributes
 * from a common source. */
#![deny(
  clippy::all,
  clippy::default_trait_access,
  clippy::expl_impl_clone_on_copy,
  clippy::if_not_else,
  clippy::needless_continue,
  clippy::unseparated_literal_suffix,
  clippy::used_underscore_binding
)]
/* We use inner modules in several places in this crate for ergonomics. */
#![allow(clippy::module_inception)]
/* It is often more clear to show that nothing is being moved. */
#![allow(clippy::match_ref_pats)]
/* Subjective style. */
#![allow(
  clippy::len_without_is_empty,
  clippy::redundant_field_names,
  clippy::too_many_arguments
)]
/* Default isn't as big a deal as people seem to think it is. */
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
/* Arc<Mutex> can be more clear than needing to grok Orderings: */
#![allow(clippy::mutex_atomic)]

use cpal::{
  self,
  traits::{DeviceTrait, HostTrait, StreamTrait},
  Stream,
};
use displaydoc::Display;
use ringbuf;
use symphonia::{
  self,
  core::{
    audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec},
    codecs::{DecoderOptions, FinalizeResult},
    conv::ConvertibleSample,
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::{Limit, MetadataOptions},
    probe::{Hint, ProbeResult},
    units::Duration,
  },
};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::console;

use std::fmt::Debug;
use std::io::{Cursor, ErrorKind};
use std::marker::Send;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
  // This provides better error messages in debug mode.
  // It's disabled in release mode so it doesn't bloat up the file size.
  #[cfg(target_arch = "wasm32")]
  console_error_panic_hook::set_once();

  Ok(())
}

#[wasm_bindgen]
pub struct Handle(Stream);

impl Handle {
  pub fn stream(&self) -> &Stream {
    &self.0
  }
}

#[wasm_bindgen]
pub fn beep() -> Handle {
  let host = cpal::default_host();
  let device = host
    .default_output_device()
    .expect("failed to find a default output device");
  let config = device.default_output_config().unwrap();

  Handle(match config.sample_format() {
    cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
    cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
    cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
  })
}

fn is_normal_eof(e: &SymphoniaError) -> bool {
  match e {
    SymphoniaError::IoError(e) => match e.kind() {
      ErrorKind::UnexpectedEof => true,
      _ => false,
    },
    _ => false,
  }
}

#[derive(Debug, Error, Display)]
pub enum SoundManipulationError {
  /// error opening stream
  OpenStreamError,
  /// error playing stream
  PlayStreamError,
  /// error closing stream
  StreamClosedError,
  /* /// io error: {0} */
  /* IoError(#[from] io::Error), */
}

/// Taken from symphonia's examples: https://github.com/pdeljanov/Symphonia/blob/8f4aaed599ba8c23aab55d1bdad65ed621a68b92/symphonia-play/src/output.rs#L15-L18.
pub trait AudioOutput {
  fn write(&mut self, audio_buf_ref: AudioBufferRef<'_>) -> Result<(), SoundManipulationError>;
  fn flush(&mut self);
  fn stream(&self) -> &cpal::Stream;
}

/// This is taken from later in that file: https://github.com/pdeljanov/Symphonia/blob/8f4aaed599ba8c23aab55d1bdad65ed621a68b92/symphonia-play/src/output.rs#L182-L187.
pub struct CpalAudioOutput;

trait AudioOutputSample:
  cpal::Sample + ConvertibleSample + RawSample + Send + Copy + Debug + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
  pub fn try_open(
    spec: SignalSpec,
    duration: Duration,
  ) -> Result<CpalAudioOutputImpl, SoundManipulationError> {
    // Get default host.
    let host = cpal::default_host();

    // Get the default audio output device.
    let device = match host.default_output_device() {
      Some(device) => device,
      _ => {
        console::error_1(&format!("failed to get default audio output device").into());
        return Err(SoundManipulationError::OpenStreamError);
      }
    };

    let config = match device.default_output_config() {
      Ok(config) => config,
      Err(err) => {
        console::error_1(
          &format!("failed to get default audio output device config: {}", err).into(),
        );
        return Err(SoundManipulationError::OpenStreamError);
      }
    };

    // Select proper playback routine based on sample format.
    match config.sample_format() {
      cpal::SampleFormat::F32 => CpalAudioOutputImpl::try_open(spec, duration, &device),
      x => unreachable!(
        "can't process anything except f32 for now: got format {:?}",
        x
      ),
      /* cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(spec, duration, &device), */
      /* cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(spec, duration, &device), */
    }
  }
}

#[wasm_bindgen]
pub struct CpalAudioOutputImpl {
  ring_buf_producer: ringbuf::Producer<f32>,
  sample_buf: SampleBuffer<f32>,
  stream: cpal::Stream,
}

impl CpalAudioOutputImpl {
  pub fn stream(&self) -> &cpal::Stream {
    &self.stream
  }
}

#[wasm_bindgen]
pub struct CpalStreamHandle(CpalAudioOutputImpl);

impl CpalStreamHandle {
  pub fn inner(&self) -> &CpalAudioOutputImpl {
    &self.0
  }
}

impl CpalAudioOutputImpl {
  pub fn try_open(
    spec: SignalSpec,
    duration: Duration,
    device: &cpal::Device,
  ) -> Result<Self, SoundManipulationError> {
    let num_channels = spec.channels.count();

    // Output audio stream config.
    let config = cpal::StreamConfig {
      channels: num_channels as cpal::ChannelCount,
      sample_rate: cpal::SampleRate(spec.rate),
      buffer_size: cpal::BufferSize::Default,
    };

    // Create a ring buffer with a capacity for up-to 200ms of audio.
    let ring_len: usize = ((200 * spec.rate as usize) / 1000) * num_channels;
    let ring_len: usize = ring_len * 1000;

    let ring_buf = ringbuf::RingBuffer::new(ring_len);
    let (ring_buf_producer, mut ring_buf_consumer) = ring_buf.split();

    let stream_result = device.build_output_stream(
      &config,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // Write out as many samples as possible from the ring buffer to the audio
        // output.
        let written = ring_buf_consumer.pop_slice(data);
        assert_eq!(written, data.len());
        /* // Mute any remaining samples. */
        /* data[written..].iter_mut().for_each(|s| *s = T::MID); */
      },
      move |err| console::error_1(&format!("audio output error: {}", err).into()),
    );

    match stream_result {
      Err(err) => {
        console::error_1(&format!("audio output stream open error: {}", err).into());
        return Err(SoundManipulationError::OpenStreamError);
      }
      Ok(stream) => {
        // Start the output stream.
        if let Err(err) = stream.play() {
          console::error_1(&format!("audio output stream play error: {}", err).into());

          return Err(SoundManipulationError::PlayStreamError);
        }

        let sample_buf = SampleBuffer::<f32>::new(duration, spec);

        Ok(CpalAudioOutputImpl {
          ring_buf_producer,
          sample_buf,
          stream,
        })
      }
    }
  }
}

impl AudioOutput for CpalAudioOutputImpl {
  fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<(), SoundManipulationError> {
    // Do nothing if there are no audio frames.
    if decoded.frames() == 0 {
      return Ok(());
    }

    // Audio samples must be interleaved for cpal. Interleave the samples in the audio
    // buffer into the sample buffer.
    self.sample_buf.copy_interleaved_ref(decoded);

    // Write all the interleaved samples to the ring buffer.
    let samples: &[f32] = self.sample_buf.samples();

    let written: usize = self.ring_buf_producer.push_slice(samples);
    assert_eq!(written, samples.len());

    Ok(())
  }

  fn flush(&mut self) {
    // Flush is best-effort, ignore the returned result.
    let _ = self.stream.pause();
  }

  fn stream(&self) -> &cpal::Stream {
    &self.stream
  }
}

#[wasm_bindgen]
pub fn play_recorded(handle: &CpalStreamHandle) {
  let inner = handle.inner();
  let stream = inner.stream();
  console::log_1(&format!("playing now!!").into());
  stream.play().unwrap();
}

#[wasm_bindgen]
pub fn examine_file(filename: &str, mime_type: &str, buf: Vec<u8>) -> CpalStreamHandle {
  console::log_1(&format!("filename: {}", filename).into());
  console::log_1(&format!("mime type: {}", mime_type).into());
  console::log_1(&format!("length of buf: {} bytes", buf.len()).into());
  let c = Cursor::new(buf);

  let known_codecs = symphonia::default::get_codecs();
  let probe = symphonia::default::get_probe();
  let stream = MediaSourceStream::new(Box::new(c), MediaSourceStreamOptions::default());

  let mut hint = Hint::new();
  hint.mime_type(mime_type);
  let format_options = FormatOptions::default();
  let metadata_options = MetadataOptions {
    limit_metadata_bytes: Limit::None,
    limit_visual_bytes: Limit::None,
  };

  let ProbeResult { mut format, .. } = probe
    .format(&hint, stream, &format_options, &metadata_options)
    .expect("failed to probe media format");
  let tracks = format.tracks();
  console::log_1(&format!("number of detected tracks: {}", tracks.len()).into());
  assert_eq!(tracks.len(), 1, "not exactly one track");
  let single_track = &tracks[0];

  let decoder_options = DecoderOptions { verify: true };
  let mut decoder = known_codecs
    .make(&single_track.codec_params, &decoder_options)
    .expect("unable to create decoder");

  console::log_1(&format!("initial codec params: {:?}", decoder.codec_params()).into());

  let mut output_stream: Option<CpalAudioOutputImpl> = None;

  loop {
    let packet = match format.next_packet() {
      Ok(packet) => packet,
      Err(e) => {
        /* Wait until we see the EOF signal. */
        if is_normal_eof(&e) {
          break;
        } else {
          panic!("received unexpected error decoding next packet: {:?}", e);
        }
      }
    };
    let audio_buf_ref = decoder.decode(&packet).expect("failed to decode packet");
    if output_stream.is_none() {
      let duration: Duration = audio_buf_ref.capacity() as _;
      let signal_spec = audio_buf_ref.spec().clone();
      console::log_1(&format!("current signal spec: {:?}", &signal_spec).into());
      let cpal_stream = match CpalAudioOutput::try_open(signal_spec, duration) {
        Ok(stream) => stream,
        Err(e) => {
          panic!("error opening cpal output stream: {}", e);
        }
      };
      output_stream.replace(cpal_stream);
    }
    if let Err(e) = output_stream
      .as_mut()
      .expect("output stream was initialized just above")
      .write(audio_buf_ref)
    {
      panic!("error writing to output stream: {}", e);
    }
  }

  if let FinalizeResult {
    verify_ok: Some(verify_ok),
  } = decoder.finalize()
  {
    assert!(
      verify_ok,
      "verification was enabled and supported, but failed!"
    );
  }

  match output_stream {
    None => panic!("no output stream was created!"),
    Some(mut stream) => {
      stream.flush();
      CpalStreamHandle(stream)
    }
  }
}

/* NB: if Handle is used instead of &Handle then the object must *immediately* be freed by calling
 * .free() in js after being used as the argument once! Otherwise complains of receiving null
 * pointer or use after move. */
#[wasm_bindgen]
pub fn rebeep(handle: &Handle) {
  let stream = handle.stream();
  console::log_1(&format!("rebeeped!").into());
  stream.play().unwrap();
}

#[wasm_bindgen]
pub fn unbeep(handle: &Handle) {
  let stream = handle.stream();
  console::log_1(&format!("unbeeped!").into());
  stream.pause().unwrap();
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream
where
  T: cpal::Sample,
{
  let sample_rate = config.sample_rate.0 as f32;
  let channels = config.channels as usize;

  // Produce a sinusoid of maximum amplitude.
  let mut sample_clock = 0f32;
  let mut next_value = move || {
    sample_clock = (sample_clock + 1.0) % sample_rate;
    (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
  };

  let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

  let stream = device
    .build_output_stream(
      config,
      move |data: &mut [T], _| write_data(data, channels, &mut next_value),
      err_fn,
    )
    .unwrap();
  stream.play().unwrap();
  stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
  T: cpal::Sample,
{
  for frame in output.chunks_mut(channels) {
    let value: T = cpal::Sample::from::<f32>(&next_sample());
    for sample in frame.iter_mut() {
      *sample = value;
    }
  }
}
