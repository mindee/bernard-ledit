//! `UniFFI` bindings for Bernard l'Édit.
//!
//! This one `cdylib` backs the Java (through `UniFFI`'s Kotlin bindings), .NET, Node.js and Ruby
//! packages; each package generates its code from it with its own generator. Integer arguments are
//! signed and range-checked here, as in the PHP binding: Kotlin turns unsigned types into inline
//! classes, which Java can't call.

mod args;
mod error;
mod image;
mod pdf;

pub use error::BernardLeditError;

uniffi::setup_scaffolding!("bernard_ledit");
