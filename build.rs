/* Copyright 2022 Danny McClanahan */
/* SPDX-License-Identifier: AGPL-3.0-only */

fn main() {
  println!("cargo:rustc-link-lib=gsl");
  println!("cargo:rustc-link-search=/home/cosmicexplorer/tools/s1/opt/spack/linux-alpine3-zen3/gcc-11.2.1/gsl-2.7-2ihmvo3b6lb37lnxonfoznqfr5ykrvye/lib")
}
