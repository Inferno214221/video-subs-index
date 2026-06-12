#![feature(iter_map_windows)]
#![feature(iterator_try_collect)]
#![feature(exit_status_error)]

#[macro_use] extern crate rocket;

pub mod serve;
pub mod generate;