//! This crate is evil and tries to show up in diagnostics.

#![crate_type = "lib"]
#![no_std]

pub mod evil_sysroot_module {}

pub trait EvilSysrootTrait {
    type Muahaha;
}

pub fn evil_sysroot_function() {}
