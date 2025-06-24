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

// Additional modules
pub mod menu;
pub mod visualizations;
pub mod text_rendering;
pub mod ray_pattern;
pub mod rendering;
pub mod mesmerise_circular;
pub mod pixel_utils;
pub mod audio_integration;
pub mod text_processor;
pub mod pythagoras;
pub mod simple_proof;
pub mod fibonacci_spiral;
pub mod particle_fountain;
pub mod audio_handler;
pub mod audio_playback;
pub mod world;

// Export primary types at the root
pub use crate::core::world::World as CoreWorld;
pub use crate::core::buffers::Buffers;
pub use crate::core::fps::FpsCounter;
pub use crate::types::{Color, Position, Velocity, World};
pub use crate::render::drawing::FrameBuffer;
