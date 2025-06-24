use stimstation::types::{WIDTH, HEIGHT};
use stimstation::text_rendering;
use winit::{
    keyboard::KeyCode,
    window::{Window, Fullscreen},
};
use winit_input_helper::WinitInputHelper;
use std::time::Instant;
pub struct App {
    start_time: Instant,
    is_fullscreen: bool,
    should_quit: bool,
}
impl App {
    pub fn new(window: &Window) -> Self {
        if let Some(monitor) = window.primary_monitor() {
            stimstation::ray_pattern::set_monitor_dimensions(&monitor);
        }        Self {
            start_time: Instant::now(),
            is_fullscreen: false,
            should_quit: false,
        }
    }
      pub fn handle_input(&mut self, input: &mut WinitInputHelper, window: &Window) {
        if input.key_pressed(KeyCode::KeyF) || input.key_pressed(KeyCode::F11) {
            self.is_fullscreen = !self.is_fullscreen;
            window.set_fullscreen(if self.is_fullscreen {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            });
            if let Some(monitor) = window.primary_monitor() {
                stimstation::ray_pattern::set_monitor_dimensions(&monitor);            }
        }
    }
      pub fn draw(&mut self, frame: &mut [u8]) {
        frame.fill(0);
        let elapsed = self.start_time.elapsed().as_secs_f32();
        stimstation::ray_pattern::draw_frame(frame, WIDTH, HEIGHT, elapsed, 0, WIDTH);
        if frame.iter().step_by(4).all(|&b| b == 0) {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let idx = 4 * (y * WIDTH + x) as usize;
                    frame[idx] = ((x + y) % 255) as u8;
                    frame[idx + 1] = ((x * 2) % 255) as u8;
                    frame[idx + 2] = ((y * 2) % 255) as u8;
                    frame[idx + 3] = 255;
                }
            }
            text_rendering::draw_text_ab_glyph(frame, "StimStation - Ray Pattern", 50.0, 50.0, [255, 255, 255, 255], WIDTH);        }
    }    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
