
// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod logging;
pub mod params;
pub mod defs;
pub mod sun2000;
pub mod dump;

pub use defs::*;
