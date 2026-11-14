// Copyright 2026 Julian Yap (@jyap808)
// SPDX-License-Identifier: GPL-2.0-or-later

//! Copy `memory.x` into `OUT_DIR` and point the linker at it.
//!
//! `cortex-m-rt` already passes `-Tlink.x`; it searches `OUT_DIR` for the
//! memory description, which is where this script places `memory.x`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
