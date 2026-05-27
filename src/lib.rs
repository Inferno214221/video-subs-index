#![feature(iter_map_windows)]
#![feature(iterator_try_collect)]

#[macro_use] extern crate rocket;

use rocket::fs::relative;

pub const CONTENT_ROOT: &str = relative!("content");

pub mod serve;
pub mod video;