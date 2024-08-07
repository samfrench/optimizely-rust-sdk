#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

// Reimport/export of structs to make them available at top-level
pub use client::Client;
pub use conversion::Conversion;
pub use decision::Decision;

// Regular modules
pub mod client;
pub mod conversion;
pub mod datafile;
pub mod decision;

#[cfg(feature = "online")]
pub mod event_api;
