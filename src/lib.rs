#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]

pub mod drivers;
pub mod kmain;
#[macro_use]
extern crate log;
