use glam::{Vec2, Vec3, Vec4};
use palette::{Hsv, Rgb, Srgb};
use std::time::Instant;

// Optimized color type using palette
pub type Color = Srgb<u8>;
pub type ColorF = Srgb<f32>;

// Optimized position and velocity types using glam
pub type Position = Vec2;
pub type Velocity = Vec2;
pub type Size = Vec2;

// Line segment with optimized storage
#[derive(Debug, Clone)]
pub struct Line {
    pub pos: [Position; 2],
    pub vel: [Velocity; 2],
    pub color: Color,
    pub width: f32,
    pub length: f32,
    pub cycle_speed: f32,
    pub cycle_offset: f32,
}

// Particle with optimized storage
#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Position,
    pub vel: Velocity,
    pub color: Color,
    pub life: f32,
    pub size: f32,
}

// Optimized world state
#[derive(Debug)]
pub struct World {
    pub lines: Vec<Line>,
    pub particles: Vec<Particle>,
    pub mouse_pos: Option<Position>,
    pub mouse_active: bool,
    pub background_color: Color,
    pub mode: VisualMode,
    pub target_line_count: usize,
    pub start_time: Instant,
}

// Visual modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualMode {
    Normal,
    Vortex,
    Waves,
    Rainbow,
}

// Active visualization side
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ActiveSide {
    Original,
    Circular,
    Full,
    RayPattern,
    Pythagoras,
    FibonacciSpiral,
    SimpleProof,
}

// FPS counter with optimized storage
#[derive(Debug)]
pub struct FpsCounter {
    frame_times: std::collections::VecDeque<Instant>,
    last_update: Instant,
    current_fps: f32,
    update_interval: std::time::Duration,
}

// Constants
pub const WIDTH: u32 = 1600;
pub const HEIGHT: u32 = 800;
pub const MAX_LINES: usize = 100;
pub const ORIGINAL_WIDTH: u32 = 800;
pub const ORIGINAL_HEIGHT: u32 = 400;

impl Line {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        let x = rng.gen_range(0.0..WIDTH as f32);
        let y = rng.gen_range(0.0..HEIGHT as f32);
        let speed = rng.gen_range(0.5..2.5);
        let length = rng.gen_range(30.0..120.0);
        
        Self {
            pos: [
                Position::new(x, y),
                Position::new(
                    x + rng.gen_range(-length/2.0..length/2.0),
                    y + rng.gen_range(-length/2.0..length/2.0)
                ),
            ],
            vel: [
                Velocity::new(rng.gen_range(-speed..speed), rng.gen_range(-speed..speed)),
                Velocity::new(rng.gen_range(-speed..speed), rng.gen_range(-speed..speed)),
            ],
            color: Color::new(
                rng.gen_range(50..200),
                rng.gen_range(50..200),
                rng.gen_range(150..255),
            ),
            width: rng.gen_range(1.0..3.5),
            length,
            cycle_speed: rng.gen_range(0.2..1.5),
            cycle_offset: rng.gen_range(0.0..10.0),
        }
    }
}

impl Particle {
    pub fn new(pos: Position, rng: &mut impl rand::Rng) -> Self {
        let speed = rng.gen_range(1.0..5.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        Self {
            pos,
            vel: Velocity::new(angle.cos() * speed, angle.sin() * speed),
            color: hsv_to_rgb(rng.gen_range(0.0..1.0), 0.9, 1.0),
            life: rng.gen_range(0.5..1.5),
            size: rng.gen_range(1.0..3.0),
        }
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_times: std::collections::VecDeque::with_capacity(100),
            last_update: Instant::now(),
            current_fps: 0.0,
            update_interval: std::time::Duration::from_millis(500),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.frame_times.push_back(now);

        while !self.frame_times.is_empty() && 
              now.duration_since(*self.frame_times.front().unwrap()).as_secs_f32() > 1.0 {
            self.frame_times.pop_front();
        }

        if now.duration_since(self.last_update) >= self.update_interval {
            self.current_fps = self.frame_times.len() as f32;
            self.last_update = now;
        }
    }

    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}

// Color conversion utilities using palette
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let hsv = Hsv::new(h * 360.0, s, v);
    let rgb: Rgb = hsv.into_color();
    Color::from_format(rgb)
}

pub fn color_to_rgba(color: Color) -> [u8; 4] {
    [color.red, color.green, color.blue, 255]
}

pub fn rgba_to_color(rgba: [u8; 4]) -> Color {
    Color::new(rgba[0], rgba[1], rgba[2])
} 