#![no_std]
#![forbid(unsafe_code)]

#[rustfmt::skip]
mod generated;
pub use generated::*;

pub mod reflection;
pub use reflection::{
    ReflectionService, MAX_DESCRIPTOR_FILES, MAX_EXTENSION_NUMBERS, MAX_REFLECTION_SERVICES,
};
