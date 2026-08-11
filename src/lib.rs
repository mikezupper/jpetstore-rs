//! The app as a library, with main.rs as a thin shell over it. This split
//! exists for one reason: integration tests in tests/ can only import from
//! a library crate. The binary was the whole program for eleven lessons;
//! the moment something outside src/ needed to build the router, the
//! library boundary earned its existence — same rule as every other
//! abstraction in this port.

pub mod auth;
pub mod db;
pub mod domain;
pub mod web;
