// StimStation core library
//
// This library provides the core functionality for visual simulations,
// audio processing, and interactive visualizations.

// Re-exports
pub mod core;
pub mod io;
pub mod render;
pub mod types;
pub mod utils;
pub mod viz;

// Export primary types at the root
pub use crate::core::world::World;
pub use crate::core::buffers::Buffers;
pub use crate::core::fps::FpsCounter;
pub use crate::types::{Color, Position, Velocity};
pub use crate::render::drawing::FrameBuffer;
