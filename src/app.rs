use crate::types::{ActiveSide, Color, FpsCounter, Buffers, World, WIDTH, HEIGHT, ORIGINAL_WIDTH, ORIGINAL_HEIGHT};
use crate::menu::Menu;
use crate::visualizations;
use crate::text_rendering;
use winit::{
    event::{MouseButton,WindowEvent},
    keyboard::KeyCode,
    window::Window,
};
use winit_input_helper::WinitInputHelper;
use std::time::Instant;
use pixels::Pixels;

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
    pub fn new() -> Self {
        let mut world = World::new();
        if let Some(monitor) = winit::window::Window::primary_monitor(&winit::event_loop::EventLoop::new().unwrap()) {
            visualizations::ray_pattern::set_monitor_dimensions(&monitor);
        }

        Self {
            world,
            menu: Menu::new(),
            active_side: ActiveSide::Full,
            start_time: Instant::now(),
            fps_counter: FpsCounter::new(),
            is_fullscreen: false,
            show_help: false,
            buffers: Buffers::new(),
            should_quit: false,
        }
    }

    pub fn handle_input(&mut self, input: &mut WinitInputHelper, window: &Window) {
        if self.menu.is_visible() {
            self.menu.handle_input(input, self.start_time.elapsed().as_secs_f32());
            if self.menu.has_made_selection() {
                self.active_side = self.menu.get_selected_visualization();
                self.menu.reset_selection();
                self.update_window_title(window);
            }
            return;
        }

        if input.key_pressed(KeyCode::Escape) {
            self.menu.toggle_visibility(self.start_time.elapsed().as_secs_f32());
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
                Some(winit::window::Fullscreen::Borderless(None))
            } else {
                None
            });
            if let Some(monitor) = window.primary_monitor() {
                visualizations::ray_pattern::set_monitor_dimensions(&monitor);
            }
        }

        if let Some(mouse_pos) = input.cursor() {
            let window_size = window.inner_size();
            if mouse_pos.0 < window_size.width as f32 / 2.0 {
                let adjusted_x = mouse_pos.0;
                let scale_x = ORIGINAL_WIDTH as f32 / (window_size.width as f32 / 2.0);
                let scale_y = ORIGINAL_HEIGHT as f32 / window_size.height as f32;
                self.world.set_mouse_pos(glam::vec2(adjusted_x * scale_x, mouse_pos.1 * scale_y));
            } else {
                self.world.mouse_pos = None;
            }
        }

        if let Some(_) = input.cursor() {
            let window_size = window.inner_size();
            if input.cursor().unwrap().0 < window_size.width as f32 / 2.0 {
                self.world.set_mouse_active(input.mouse_held(MouseButton::Left));
            } else {
                self.world.set_mouse_active(false);
            }
        }

        if input.key_pressed(KeyCode::KeyE) {
            let center_x = ORIGINAL_WIDTH as f32 / 2.0;
            let center_y = HEIGHT as f32 / 2.0;
            self.world.create_explosion(glam::vec2(center_x, center_y), 200);
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

        let new_active_side = match () {
            _ if input.key_pressed(KeyCode::Digit1) => Some(ActiveSide::Original),
            _ if input.key_pressed(KeyCode::Digit2) => Some(ActiveSide::Circular),
            _ if input.key_pressed(KeyCode::Digit3) => Some(ActiveSide::Full),
            _ if input.key_pressed(KeyCode::Digit4) => Some(ActiveSide::RayPattern),
            _ if input.key_pressed(KeyCode::Digit5) => Some(ActiveSide::Pythagoras),
            _ if input.key_pressed(KeyCode::Digit6) => Some(ActiveSide::FibonacciSpiral),
            _ if input.key_pressed(KeyCode::Digit7) => Some(ActiveSide::SimpleProof),
            _ if input.key_pressed(KeyCode::Digit8) => Some(ActiveSide::Combined),
            _ => None,
        };

        if let Some(new_side) = new_active_side {
            self.active_side = new_side;
            self.update_window_title(window);
        }
    }

    pub fn update(&mut self) {
        self.world.update();
        self.menu.update(self.start_time.elapsed().as_secs_f32());
        self.fps_counter.update();
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        self.buffers.clear();
        let elapsed = self.start_time.elapsed().as_secs_f32();

        match self.active_side {
            ActiveSide::Original => visualizations::draw_original(frame, &self.world, &mut self.buffers.original),
            ActiveSide::Circular => visualizations::draw_circular_with_buffer(frame, elapsed, &mut self.buffers.circular),
            ActiveSide::Full => visualizations::draw_full_screen_with_buffer(frame, &self.world, elapsed, &mut self.buffers),
            ActiveSide::RayPattern => visualizations::draw_ray_pattern(frame, elapsed),
            ActiveSide::Pythagoras => visualizations::pythagoras::draw_frame(frame, elapsed),
            ActiveSide::FibonacciSpiral => visualizations::fibonacci_spiral::draw_frame(frame, elapsed),
            ActiveSide::SimpleProof => visualizations::simple_proof::draw_frame(frame, elapsed),
            ActiveSide::Combined => visualizations::draw_all_visualizations(frame, &self.world, elapsed),
        }

        let fps_text = format!("FPS: {:.1}", self.fps_counter.fps());
        text_rendering::draw_text_ab_glyph(frame, &fps_text, 10.0, (HEIGHT - 30) as f32, [255, 255, 0, 255], WIDTH);

        if self.show_help {
            text_rendering::draw_keyboard_guide(frame, WIDTH);
        }

        if self.menu.is_visible() {
            self.menu.render(frame, WIDTH, HEIGHT);
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    fn update_window_title(&self, window: &Window) {
        let title = match self.active_side {
            ActiveSide::Original => "StimStation - Original Visualization".to_string(),
            ActiveSide::Circular => "StimStation - Circular Visualization".to_string(),
            ActiveSide::Full => "StimStation - All Visualizations (Grid)".to_string(),
            ActiveSide::RayPattern => "StimStation - Ray Pattern".to_string(),
            ActiveSide::Pythagoras => "StimStation - Pythagoras Theorem".to_string(),
            ActiveSide::FibonacciSpiral => "StimStation - Fibonacci Spiral".to_string(),
            ActiveSide::SimpleProof => "StimStation - Simple Proof".to_string(),
            ActiveSide::Combined => "StimStation - Combined Experience".to_string(),
        };
        window.set_title(&title);
    }
}
