pub mod algorithms;
pub mod audio;
pub mod core;
pub mod graphics;
pub mod physics;
pub mod text;

// Re-export commonly used types and modules
pub use core::integration;
pub use core::orchestrator;
pub use core::types;

// App module - integrates with the orchestrator
pub mod app {
    use crate::integration;
    use crate::orchestrator;
    use crate::types::{HEIGHT, WIDTH};
    use std::sync::Arc;
    use std::time::Instant;
    use winit::keyboard::KeyCode;

    pub struct App {
        quit: bool,
        start_time: Instant,
    }

    impl App {
        pub fn new(window: &Arc<winit::window::Window>) -> Self {
            // Set monitor dimensions for scaling
            if let Some(monitor) = window.current_monitor() {
                integration::set_monitor_dimensions(&monitor);
            }

            Self {
                quit: false,
                start_time: Instant::now(),
            }
        }

        pub fn draw(&mut self, frame: &mut [u8]) {
            let time = self.start_time.elapsed().as_secs_f32();
            orchestrator::draw_frame(frame, WIDTH, HEIGHT, time, 0, WIDTH);
        }

        pub fn should_quit(&self) -> bool {
            self.quit
        }

        pub fn quit(&mut self) {
            self.quit = true;
        }
        pub fn handle_input(
            &mut self,
            input: &mut winit_input_helper::WinitInputHelper,
            _window: &winit::window::Window,
        ) {
            // Add input handling for physics forces, etc.
            if input.key_pressed(KeyCode::Escape) {
                self.quit();
            }

            // Toggle white noise with '9' key
            if input.key_pressed(KeyCode::Digit9) {
                let enabled = !crate::audio::audio_playback::is_white_noise_enabled();
                crate::audio::audio_playback::set_white_noise_enabled(enabled);
                if enabled {
                    println!("White noise enabled");
                } else {
                    println!("White noise disabled");
                }
            }

            // Example: Add force to balls with arrow keys
            if input.key_held(KeyCode::ArrowLeft) {
                crate::physics::physics::apply_force_yellow(-0.1, 0.0);
            }
            if input.key_held(KeyCode::ArrowRight) {
                crate::physics::physics::apply_force_yellow(0.1, 0.0);
            }
            if input.key_held(KeyCode::ArrowUp) {
                crate::physics::physics::apply_force_yellow(0.0, -0.1);
            }
            if input.key_held(KeyCode::ArrowDown) {
                crate::physics::physics::apply_force_yellow(0.0, 0.1);
            }
        }
    }
}
