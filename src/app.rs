use stimstation::types::{ActiveSide, WIDTH, HEIGHT, ORIGINAL_WIDTH, ORIGINAL_HEIGHT, Buffers};
use stimstation::FpsCounter;
use stimstation::types::World;
use stimstation::menu::Menu;
use stimstation::visualizations;
use stimstation::text_rendering;

use winit::{
    event::MouseButton,
    keyboard::KeyCode,
    window::{Window, Fullscreen},
};
use winit_input_helper::WinitInputHelper;

use glam::vec2;

use std::time::Instant;

/// High‑level application wrapper around all StimStation visualisations
pub struct App {
    world: World,
    menu: Menu,
    active_side: ActiveSide,
    start_time: Instant,
    fps_counter: FpsCounter,
    is_fullscreen: bool,
    show_help: bool,
    buffers: Buffers,
    should_quit: bool,
}

impl App {
    /// Creates a fresh `App` instance and initialises the ray‑pattern helper
    pub fn new(window: &Window) -> Self {
        let world = World::new();

        if let Some(monitor) = window.primary_monitor() {
            stimstation::ray_pattern::set_monitor_dimensions(&monitor);
        }

        Self {
            world,
            menu: Menu::new(),
            active_side: ActiveSide::Circular, // default = simplest
            start_time: Instant::now(),
            fps_counter: FpsCounter::new(),
            is_fullscreen: false,
            show_help: false,
            buffers: Buffers::new(),
            should_quit: false,
        }
    }

    /// Dispatches window / keyboard / mouse input events
    pub fn handle_input(&mut self, input: &mut WinitInputHelper, window: &Window) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        // ── 1. Handle menu interaction first ───────────────────────────────
        if self.menu.is_visible() {
            self.menu
                .handle_input(input, self.start_time.elapsed().as_secs_f32());

            if self.menu.has_made_selection() {
                self.active_side = self.menu.get_selected_visualization();
                self.menu.reset_selection();
                self.update_window_title(window);
            }
            return;
        }

        // ── 2. Global hot‑keys ─────────────────────────────────────────────
        if input.key_pressed(KeyCode::Escape) {
            self.menu
                .toggle_visibility(self.start_time.elapsed().as_secs_f32());
        }
        if input.key_pressed(KeyCode::Space) {
            self.world.toggle_mode();
            self.update_window_title(window);
        }
        if input.key_pressed(KeyCode::KeyH) {
            self.show_help = !self.show_help;
            self.update_window_title(window);
        }
        if input.key_pressed(KeyCode::KeyF) || input.key_pressed(KeyCode::F11) {
            self.is_fullscreen = !self.is_fullscreen;
            window.set_fullscreen(if self.is_fullscreen {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            });

            if let Some(monitor) = window.primary_monitor() {
                stimstation::ray_pattern::set_monitor_dimensions(&monitor);
            }
        }

        // ── 3. Mouse handling ──────────────────────────────────────────────
        if let Some(mouse_pos) = input.cursor() {
            let win = window.inner_size();
            if mouse_pos.0 < win.width as f32 / 2.0 {
                // Only interact inside the original‑visualisation quadrant
                let scale_x = ORIGINAL_WIDTH as f32 / (win.width as f32 / 2.0);
                let scale_y = ORIGINAL_HEIGHT as f32 / win.height as f32;
                self.world.set_mouse_pos(vec2(mouse_pos.0 * scale_x, mouse_pos.1 * scale_y));
            } else {
                self.world.mouse_pos = None;
            }
        }

        if let Some(cursor) = input.cursor() {
            let win = window.inner_size();
            let over_left = cursor.0 < win.width as f32 / 2.0;
            self.world
                .set_mouse_active(over_left && input.mouse_held(MouseButton::Left));
        }

        // ── 4. World‑level interactions ───────────────────────────────────
        if input.key_pressed(KeyCode::KeyE) {
            let centre = vec2(ORIGINAL_WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
            self.world.create_explosion(centre, 200);
        }
        if input.mouse_pressed(MouseButton::Right) {
            if let Some(pos) = self.world.mouse_pos {
                self.world.create_explosion(pos, 100);
            }
        }
        if input.key_pressed(KeyCode::Equal) {
            self.world.add_lines(10);
            self.update_window_title(window);
        }
        if input.key_pressed(KeyCode::Minus) {
            self.world.remove_lines(10);
            self.update_window_title(window);
        }

        // ── 5. Quick numeric shortcuts to switch visualisations ────────────
        let new_active_side = if input.key_pressed(KeyCode::Digit1) {
            Some(ActiveSide::Original)
        } else if input.key_pressed(KeyCode::Digit2) {
            Some(ActiveSide::Circular)
        } else if input.key_pressed(KeyCode::Digit3) {
            Some(ActiveSide::Full)
        } else if input.key_pressed(KeyCode::Digit4) {
            Some(ActiveSide::RayPattern)
        } else if input.key_pressed(KeyCode::Digit5) {
            Some(ActiveSide::Pythagoras)
        } else if input.key_pressed(KeyCode::Digit6) {
            Some(ActiveSide::FibonacciSpiral)
        } else if input.key_pressed(KeyCode::Digit7) {
            Some(ActiveSide::SimpleProof)
        } else if input.key_pressed(KeyCode::Digit8) {
            Some(ActiveSide::Combined)
        } else {
            None
        };

        if let Some(side) = new_active_side {
            self.active_side = side;
            self.update_window_title(window);
        }
    }

    /// Advances simulation state once per frame
    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.world.update();
        self.menu.update(self.start_time.elapsed().as_secs_f32());
        self.fps_counter.update();
    }

    /// Renders the frame into the RGBA pixel buffer returned by `pixels()`
    pub fn draw(&mut self, frame: &mut [u8]) {
        frame.fill(0); // clear to black

        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Menu supersedes all viz output
        if self.menu.is_visible() {
            self.menu.render(frame, WIDTH, HEIGHT);
            return;
        }

        self.buffers.clear();

        match self.active_side {
            ActiveSide::Original => visualizations::draw_original_with_buffer(frame, &self.world, &mut self.buffers.original),
            ActiveSide::Circular => visualizations::draw_circular_with_buffer(frame, elapsed, &mut self.buffers.circular),
            ActiveSide::Full => visualizations::draw_full_screen_with_buffer(frame, &self.world, elapsed, &mut self.buffers),
            ActiveSide::RayPattern => stimstation::ray_pattern::draw_frame(frame, WIDTH, HEIGHT, elapsed, 0, WIDTH),
            ActiveSide::Pythagoras => visualizations::draw_pythagoras_frame(frame, elapsed),
            ActiveSide::FibonacciSpiral => visualizations::draw_fibonacci_frame(frame, elapsed),
            ActiveSide::SimpleProof => visualizations::draw_simple_proof_frame(frame, elapsed),
            ActiveSide::Combined => visualizations::draw_all_visualizations(frame, &self.world, elapsed),
        }

        // HUD / overlay text ------------------------------------------------
        let fps_text = format!("FPS: {:.1}", self.fps_counter.fps());
        text_rendering::draw_text_ab_glyph(frame, &fps_text, 10.0, (HEIGHT - 30) as f32, [255, 255, 0, 255], WIDTH);

        if self.show_help {
            text_rendering::draw_keyboard_guide(frame, WIDTH);
        }

        // Safety‑net: if we somehow drew nothing, show a colourful pattern
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
            text_rendering::draw_text_ab_glyph(frame, "StimStation - Test Pattern", 50.0, 50.0, [255, 255, 255, 255], WIDTH);
        }
    }

    // ── Control‑flow helpers ───────────────────────────────────────────────
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    fn update_window_title(&self, window: &Window) {
        let title = match self.active_side {
            ActiveSide::Original => "StimStation - Original Visualization",
            ActiveSide::Circular => "StimStation - Circular Visualization",
            ActiveSide::Full => "StimStation - All Visualizations (Grid)",
            ActiveSide::RayPattern => "StimStation - Ray Pattern",
            ActiveSide::Pythagoras => "StimStation - Pythagoras Theorem",
            ActiveSide::FibonacciSpiral => "StimStation - Fibonacci Spiral",
            ActiveSide::SimpleProof => "StimStation - Simple Proof",
            ActiveSide::Combined => "StimStation - Combined Experience",
        };
        
        window.set_title(title);
    }
}
