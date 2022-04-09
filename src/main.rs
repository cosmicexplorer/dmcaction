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

use rand;
use rgsl::{
  sort::vectors::sort_index,
  types::wavelet_transforms::{Wavelet, WaveletType, WaveletWorkspace},
  wavelet_transforms::one_dimension::{transform_forward, transform_inverse},
  Value,
};

const N: usize = 256;
const NC: usize = 20;

fn main() {
  let mut orig_data: [f64; N] = [0.0; N];
  for i in 0..N {
    orig_data[i] = rand::random();
  }
  let mut data: [f64; N] = orig_data;
  let mut abscoeff: [f64; N] = [0.0; N];
  let mut p: [usize; N] = [0; N];

  let wavelet = Wavelet::new(WaveletType::daubechies(), 4).expect("no we have enough memory");
  let mut workspace = WaveletWorkspace::new(N).expect("no we have enough memory");

  let result = transform_forward(&wavelet, &mut data, 1, N, &mut workspace);
  assert_eq!(result, Value::Success);

  for i in 0..N {
    abscoeff[i] = data[i].abs();
  }

  sort_index(&mut p, &mut abscoeff, 1, N);

  for i in 0..(N - NC) {
    data[p[i]] = 0.0;
  }

  let result = transform_inverse(&wavelet, &mut data, 1, N, &mut workspace);
  assert_eq!(result, Value::Success);

  for i in 0..N {
    println!("{} {}", orig_data[i], data[i]);
  }
}
