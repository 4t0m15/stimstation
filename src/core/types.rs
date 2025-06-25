use glam::Vec2;
use palette::{Hsv, IntoColor, Srgb};
use rand::prelude::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
pub type Color = Srgb<u8>;
pub type Position = Vec2;
pub type Velocity = Vec2;
pub const WIDTH: u32 = 1600;
pub const HEIGHT: u32 = 800;
pub const MAX_LINES: usize = 100;
pub const ORIGINAL_WIDTH: u32 = 800;
pub const ORIGINAL_HEIGHT: u32 = 400;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualMode {
    Normal,
    Vortex,
    Waves,
    Rainbow,
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ActiveSide {
    Original,
    Circular,
    Full,
    RayPattern,
    Pythagoras,
    FibonacciSpiral,
    SimpleProof,
    Combined,
}
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
pub type SimplePos = (f32, f32);
#[derive(Debug)]
pub struct SimpleLine {
    pub pos: [SimplePos; 2],
    pub vel: [SimplePos; 2],
    pub color: SimpleColor,
    pub width: f32,
    pub length: f32,
    pub cycle_speed: f32,
    pub cycle_offset: f32,
}
#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Position,
    pub vel: Velocity,
    pub color: Color,
    pub life: f32,
    pub size: f32,
}
#[derive(Debug)]
pub struct SimpleParticle {
    pub pos: SimplePos,
    pub vel: SimplePos,
    pub color: SimpleColor,
    pub life: f32,
    pub size: f32,
}
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
pub type SimpleColor = [u8; 3];
#[derive(Debug)]
pub struct SimpleWorld {
    pub lines: Vec<SimpleLine>,
    pub rng: ThreadRng,
    pub start_time: Instant,
    pub mouse_pos: Option<SimplePos>,
    pub mouse_active: bool,
    pub background_color: SimpleColor,
    pub mode: VisualMode,
    pub particles: Vec<SimpleParticle>,
    pub target_line_count: usize,
}
#[derive(Debug)]
pub struct FpsCounter {
    pub frame_times: VecDeque<Instant>,
    pub last_update: Instant,
    pub current_fps: f32,
    pub update_interval: Duration,
}
#[derive(Debug)]
pub struct Buffers {
    pub original: Vec<u8>,
    pub circular: Vec<u8>,
    pub full: Vec<u8>,
}
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
                    x + rng.gen_range(-length / 2.0..length / 2.0),
                    y + rng.gen_range(-length / 2.0..length / 2.0),
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
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let hsv = Hsv::new(h * 360.0, s, v);
    let rgb: Srgb = hsv.into_color();
    Color::from_format(rgb)
}
pub fn color_to_rgba(color: Color) -> [u8; 4] {
    [color.red, color.green, color.blue, 255]
}
pub fn rgba_to_color(rgba: [u8; 4]) -> Color {
    Color::new(rgba[0], rgba[1], rgba[2])
}
pub fn simple_hsv_to_rgb(h: f32, s: f32, v: f32) -> SimpleColor {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}
