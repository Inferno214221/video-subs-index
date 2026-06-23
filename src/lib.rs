#![feature(iter_map_windows)]
#![feature(iterator_try_collect)]
#![feature(exit_status_error)]
#![feature(clone_from_ref)]

#[macro_use] extern crate rocket;

pub mod serve;
pub mod generate;