/* Copyright 2022 Danny McClanahan */
/* SPDX-License-Identifier: AGPL-3.0-only */

//! ???

/* Turn all warnings into errors! */
#![deny(warnings)]
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
  traits::{DeviceTrait, HostTrait, StreamTrait},
  Stream,
};
use wasm_bindgen::prelude::*;
use web_sys::console;

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
