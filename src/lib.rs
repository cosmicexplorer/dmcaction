/* Copyright 2022-2023 Danny McClanahan */
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
use symphonia::{
  self,
  core::{
    audio::SampleBuffer,
    codecs::{DecoderOptions, FinalizeResult},
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::{Limit, MetadataOptions},
    probe::{Hint, ProbeResult},
    units::Duration,
  },
};
use wasm_bindgen::prelude::*;
use wav;
use web_sys::console;

use std::{
  io::{Cursor, ErrorKind},
  mem,
};

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

 #[wasm_bindgen]
pub fn log_result() {
  console::log_1(&format!("HUH WOW").into());
  /* todo!("log result") */
  /* use ffmpeg::ffmpeg_sys::bindings; */
  /* console::log_1( */
  /*   &format!( */
  /*     "LIBAVUTIL_VERSION_MAJOR = {}", */
  /*     bindings::LIBAVUTIL_VERSION_MAJOR */
  /*   ) */
  /*   .into(), */
  /* ); */
  /* console::log_1( */
  /*   &format!("avutil_version = {:?}", unsafe { */
  /*     bindings::avutil_version() */
  /*   }) */
  /*   .into(), */
  /* ); */
}

#[wasm_bindgen]
pub fn examine_file(filename: &str, mime_type: &str, buf: Vec<u8>) -> Vec<u8> {
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

  let mut wav_header: Option<wav::header::Header> = None;
  let mut floats: Vec<f32> = Vec::new();
  let mut result: Cursor<Vec<u8>> = Cursor::new(Vec::new());

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
    let duration: Duration = audio_buf_ref.capacity() as _;
    let signal_spec = audio_buf_ref.spec().clone();
    if wav_header.is_none() {
      wav_header.replace(wav::header::Header::new(
        wav::header::WAV_FORMAT_IEEE_FLOAT,
        signal_spec.channels.count() as u16,
        signal_spec.rate,
        (mem::size_of::<f32>() * 8) as u16,
      ));
    }
    let mut sample_buffer: SampleBuffer<f32> = SampleBuffer::new(duration, signal_spec);
    sample_buffer.copy_planar_ref(audio_buf_ref);
    floats.extend(sample_buffer.samples());
  }

  if let Some(wav_header) = wav_header {
    let wav_bit_depth = wav::bit_depth::BitDepth::ThirtyTwoFloat(floats);
    wav::write(wav_header, &wav_bit_depth, &mut result).expect("writing should not fail");
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

  result.into_inner()
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
