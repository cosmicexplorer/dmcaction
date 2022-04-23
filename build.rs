/* Copyright 2022 Danny McClanahan */
/* SPDX-License-Identifier: AGPL-3.0-only */

#![deny(warnings)]
// Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be copied and pasted across crates, but there doesn't appear to be a way to include inner attributes from a common source.
#![deny(
  clippy::all,
  clippy::default_trait_access,
  clippy::expl_impl_clone_on_copy,
  clippy::if_not_else,
  clippy::needless_continue,
  clippy::unseparated_literal_suffix,
  clippy::used_underscore_binding
)]
// It is often more clear to show that nothing is being moved.
#![allow(clippy::match_ref_pats)]
// Subjective style.
#![allow(clippy::redundant_field_names, clippy::too_many_arguments)]
// Default isn't as big a deal as people seem to think it is.
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
// Arc<Mutex> can be more clear than needing to grok Orderings:
#![allow(clippy::mutex_atomic)]

use cc;
use walkdir;

use std::path::Path;

/* Build script taken from this github comment from 2020: https://github.com/rustwasm/team/issues/291#issuecomment-645492619 */
fn main() {
  let wasm_object_file = "/home/cosmicexplorer/tools/emscripten/f.o.emcc";
  mark_for_change_detection(wasm_object_file.as_ref());
  cc::Build::new()
    .object(wasm_object_file)
    .shared_flag(true)
    .static_flag(false)
    .compile("f");
}

fn mark_for_change_detection(path: &Path) {
  // Restrict re-compilation check to just our input files.
  // See: http://doc.crates.io/build-script.html#outputs-of-the-build-script
  if !path.exists() {
    panic!(
      "Cannot mark non-existing path for change detection: {}",
      path.display()
    );
  }
  for file in walkdir::WalkDir::new(path) {
    println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
  }
}
