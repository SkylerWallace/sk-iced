//! Local widget motion: accepted actions are dispatched immediately.
pub mod feedback;
pub mod highlight;
pub mod keyed;
pub mod primitives;
pub mod responsive;
pub mod reveal;
pub type Renderer = iced_renderer::Renderer;
pub type Element<'a, M> = iced_core::Element<'a, M, iced_core::Theme, Renderer>;

pub mod drawer;
