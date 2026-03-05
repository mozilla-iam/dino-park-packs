// DEBT: This is required because of diesel.
// Prior art: https://github.com/mozilla-services/syncstorage-rs/blob/f2e18e2df9e166a66d00ecf41b89806b905b86d6/syncstorage-mysql/src/lib.rs#L1
//
// It's not clear to me if there's a future where we'll be able to disable this
// lint.
#![allow(non_local_definitions)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate dino_park_guard;

pub mod api;
pub mod cis;
pub mod db;
pub mod error;
pub mod healthz;
pub mod import;
pub mod mail;
pub mod rules;
pub mod settings;
pub mod user;
pub mod utils;
