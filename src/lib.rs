//! Reusable Iced components, themes and policy-controlled motion.
pub mod motion;
#[cfg(feature = "theme")]
pub mod palette;
#[cfg(feature = "theme")]
pub mod theme;

#[cfg(feature = "animations")]
pub mod animation;

#[cfg(feature = "animations")]
pub mod components;

pub mod design;
#[cfg(feature = "icons")]
pub mod icons;
#[cfg(feature = "theme")]
pub mod typography;

#[cfg(feature = "theme")]
pub mod visual;

#[cfg(feature = "theme")]
pub mod themes;

/// Standard element and renderer aliases for the supported Iced configuration.
#[cfg(feature = "animations")]
pub use animation::{Element, Renderer};
