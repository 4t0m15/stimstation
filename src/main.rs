use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use std::sync::Arc;
use winit::{
	dpi::LogicalSize,
	event::{MouseButton},
	keyboard::KeyCode,
	event_loop::EventLoop,
	window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;
use std::time::{Duration, Instant};

// Import our circular visuals implementation
mod mesmerise_circular;
mod ray_pattern;

// Combined window dimensions
const WIDTH: u32 = 1600;
const HEIGHT: u32 = 800;  // Increased height for more visualizations
const MAX_LINES: usize = 100;

// Original visualization dimensions
const ORIGINAL_WIDTH: u32 = 800;
const ORIGINAL_HEIGHT: u32 = 400;  // Reduced height for original

// Flag to control which part of the visualization is active
#[derive(Debug, PartialEq)]
enum ActiveSide {
	Original,  // Original lines only
	Circular,  // Circular only
	Full,      // Full screen with all visualizations
	RayPattern // Only show the ray pattern
}

#[derive(Debug)]
struct Line {
	pos: [(f32, f32); 2], // [(x1, y1), (x2, y2)]
	vel: [(f32, f32); 2], // [(vx1, vy1), (vx2, vy2)]
	color: [u8; 3],       // [r, g, b]
	width: f32,
	length: f32,          // Target line length
	cycle_speed: f32,     // How fast this line's color cycles
	cycle_offset: f32,    // Individual offset for color cycling
}

impl Line {
	fn new(rng: &mut impl Rng) -> Self {
		let x = rng.gen_range(0.0..WIDTH as f32);
		let y = rng.gen_range(0.0..HEIGHT as f32);
		let speed = rng.gen_range(0.5..2.5);  // Increased speed range
		let length = rng.gen_range(30.0..120.0);  // Line length
		
		Self {
			pos: [
				(x, y),
				(x + rng.gen_range(-length/2.0..length/2.0), y + rng.gen_range(-length/2.0..length/2.0)),
			],
			vel: [
				(rng.gen_range(-speed..speed), rng.gen_range(-speed..speed)),
				(rng.gen_range(-speed..speed), rng.gen_range(-speed..speed)),
			],
			color: [
				rng.gen_range(50..200),
				rng.gen_range(50..200),
				rng.gen_range(150..255),
			],
			width: rng.gen_range(1.0..3.5),  // Slightly thicker lines
			length,
			cycle_speed: rng.gen_range(0.2..1.5),  // Color cycling speed
			cycle_offset: rng.gen_range(0.0..10.0),  // Random offset
		}
	}
	
	fn update(&mut self, rng: &mut impl Rng, time: f32, mouse_pos: Option<(f32, f32)>) {
		// Color cycling based on time
		let hue = (time * self.cycle_speed + self.cycle_offset) % 1.0;
		self.color = hsv_to_rgb(hue, 0.8, 0.9);
		
		// Add mouse attraction if mouse position is available
		if let Some((mx, my)) = mouse_pos {
			// Calculate distance to mouse
			let dist1 = ((self.pos[0].0 - mx).powi(2) + (self.pos[0].1 - my).powi(2)).sqrt();
			let dist2 = ((self.pos[1].0 - mx).powi(2) + (self.pos[1].1 - my).powi(2)).sqrt();
			
			// Apply attraction force (inverse to distance)
			if dist1 > 5.0 {  // Avoid extreme acceleration when very close
				let force = 40.0 / dist1;
				let dx = (mx - self.pos[0].0) / dist1 * force;
				let dy = (my - self.pos[0].1) / dist1 * force;
				self.vel[0].0 = (self.vel[0].0 * 0.95 + dx * 0.05).clamp(-3.0, 3.0);
				self.vel[0].1 = (self.vel[0].1 * 0.95 + dy * 0.05).clamp(-3.0, 3.0);
			}
			
			if dist2 > 5.0 {
				let force = 40.0 / dist2;
				let dx = (mx - self.pos[1].0) / dist2 * force;
				let dy = (my - self.pos[1].1) / dist2 * force;
				self.vel[1].0 = (self.vel[1].0 * 0.95 + dx * 0.05).clamp(-3.0, 3.0);
				self.vel[1].1 = (self.vel[1].1 * 0.95 + dy * 0.05).clamp(-3.0, 3.0);
			}
		}
		
		// Update positions
		for i in 0..2 {
			self.pos[i].0 += self.vel[i].0;
			self.pos[i].1 += self.vel[i].1;
			
			// Bounce off edges with energy loss
			if self.pos[i].0 < 0.0 || self.pos[i].0 >= WIDTH as f32 {
				self.vel[i].0 *= -0.9;  // 10% energy loss on bounce
				self.pos[i].0 = self.pos[i].0.clamp(0.0, WIDTH as f32 - 1.0);
			}
			if self.pos[i].1 < 0.0 || self.pos[i].1 >= HEIGHT as f32 {
				self.vel[i].1 *= -0.9;  // 10% energy loss on bounce
				self.pos[i].1 = self.pos[i].1.clamp(0.0, HEIGHT as f32 - 1.0);
			}
		}
		
		// Maintain roughly constant line length by adding a spring force
		let dx = self.pos[1].0 - self.pos[0].0;
		let dy = self.pos[1].1 - self.pos[0].1;
		let current_length = (dx * dx + dy * dy).sqrt();
		let difference = current_length - self.length;
		
		if current_length > 0.1 {  // Avoid division by zero
			let force = difference * 0.01;  // Spring force
			let dx_norm = dx / current_length;
			let dy_norm = dy / current_length;
			
			// Apply forces in opposite directions
			self.vel[0].0 += dx_norm * force;
			self.vel[0].1 += dy_norm * force;
			self.vel[1].0 -= dx_norm * force;
			self.vel[1].1 -= dy_norm * force;
		}
		
		// Occasionally change velocity with small random adjustments
		if rng.gen_bool(0.02) {
			for i in 0..2 {
				self.vel[i].0 = (self.vel[i].0 + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
				self.vel[i].1 = (self.vel[i].1 + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
			}
		}
	}
}

// Convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
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

// New particle struct for explosion effects
#[derive(Debug)]
struct Particle {
	pos: (f32, f32),
	vel: (f32, f32),
	color: [u8; 3],
	life: f32,  // Remaining lifetime in seconds
	size: f32,
}

impl Particle {
	fn new(x: f32, y: f32, rng: &mut impl Rng) -> Self {
		let speed = rng.gen_range(1.0..5.0);
		let angle = rng.gen_range(0.0..std::f32::consts::TAU);
		
		Self {
			pos: (x, y),
			vel: (angle.cos() * speed, angle.sin() * speed),
			color: hsv_to_rgb(rng.gen_range(0.0..1.0), 0.9, 1.0),
			life: rng.gen_range(0.5..1.5),
			size: rng.gen_range(1.0..3.0),
		}
	}
	
	fn update(&mut self, dt: f32) -> bool {
		// Update position
		self.pos.0 += self.vel.0;
		self.pos.1 += self.vel.1;
		
		// Add some gravity
		self.vel.1 += 0.1;
		
		// Reduce life
		self.life -= dt;
		
		// Return true if still alive
		self.life > 0.0
	}
}

struct World {
	lines: Vec<Line>,
	rng: rand::rngs::ThreadRng,
	start_time: Instant,
	mouse_pos: Option<(f32, f32)>,
	mouse_active: bool,
	background_color: [u8; 3],
	mode: VisualMode,
	particles: Vec<Particle>,
	target_line_count: usize,  // Desired number of lines
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum VisualMode {
	Normal,
	Vortex,
	Waves,
	Rainbow,  // New rainbow mode
}

impl World {
	fn new() -> Self {
		let mut rng = rand::thread_rng();
		Self {
			lines: (0..MAX_LINES).map(|_| Line::new(&mut rng)).collect(),
			rng,
			start_time: Instant::now(),
			mouse_pos: None,
			mouse_active: false,
			background_color: [0, 0, 0],
			mode: VisualMode::Normal,
			particles: Vec::with_capacity(500),
			target_line_count: MAX_LINES,
		}
	}
	
	fn create_explosion(&mut self, x: f32, y: f32, count: usize) {
		for _ in 0..count {
			self.particles.push(Particle::new(x, y, &mut self.rng));
		}
	}

	fn update(&mut self) {
		let elapsed = self.start_time.elapsed().as_secs_f32();
		
		// Update particles
		let dt = 1.0 / 60.0; // Assume 60 FPS for physics
		self.particles.retain_mut(|p| p.update(dt));
		
		// Slowly cycle background color
		let bg_hue = (elapsed * 0.02) % 1.0;
		self.background_color = hsv_to_rgb(bg_hue, 0.5, 0.1); // Low value for dark background
		
		// Apply different effects based on mode
		match self.mode {
			VisualMode::Normal => {
				// Normal update with mouse interaction
				for line in &mut self.lines {
					line.update(&mut self.rng, elapsed, self.mouse_pos);
				}
			},
			VisualMode::Vortex => {
				// Create a vortex effect around the center or mouse
				let center = self.mouse_pos.unwrap_or((WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0));
				
				for line in &mut self.lines {
					// First update normally
					line.update(&mut self.rng, elapsed, None);
					
					// Then add vortex effect
					for i in 0..2 {
						let dx = line.pos[i].0 - center.0;
						let dy = line.pos[i].1 - center.1;
						let dist = (dx * dx + dy * dy).sqrt();
						
						if dist > 5.0 {
							// Calculate perpendicular force (for rotation)
							let force = 300.0 / dist;
							let vx = -dy / dist * force;
							let vy = dx / dist * force;
							
							// Apply vortex force
							line.vel[i].0 = (line.vel[i].0 * 0.95 + vx * 0.05).clamp(-5.0, 5.0);
							line.vel[i].1 = (line.vel[i].1 * 0.95 + vy * 0.05).clamp(-5.0, 5.0);
						}
					}
				}
			},
			VisualMode::Waves => {
				// Create wave-like motion
				let time_factor = elapsed * 2.0;
				
				for line in &mut self.lines {
					// Normal update first
					line.update(&mut self.rng, elapsed, None);
					
					// Add wave effect - vary speed based on position
					for i in 0..2 {
						let wave_x = (line.pos[i].0 / 100.0 + time_factor).sin() * 0.2;
						let wave_y = (line.pos[i].1 / 80.0 + time_factor * 0.7).cos() * 0.2;
						
						line.vel[i].0 = (line.vel[i].0 * 0.95 + wave_x).clamp(-3.0, 3.0);
						line.vel[i].1 = (line.vel[i].1 * 0.95 + wave_y).clamp(-3.0, 3.0);
					}
				}
			},
			VisualMode::Rainbow => {
				// Create rainbow effect with synchronized colors
				let global_hue = elapsed * 0.3 % 1.0; // Global hue that changes over time
				let line_count = self.lines.len() as f32; // Calculate once before the loop
				
				for (i, line) in self.lines.iter_mut().enumerate() {
					// Normal update first
					line.update(&mut self.rng, elapsed, None);
					
					// Set rainbow color pattern with offset based on line index
					let line_offset = (i as f32 / line_count) * 0.5;
					let hue = (global_hue + line_offset) % 1.0;
					line.color = hsv_to_rgb(hue, 0.9, 0.9);
					
					// Add swirling motion
					for j in 0..2 {
						let angle = elapsed * 0.2 + (i as f32 * 0.01);
						let swirl_x = angle.sin() * 0.2;
						let swirl_y = angle.cos() * 0.2;
						
						line.vel[j].0 = (line.vel[j].0 * 0.95 + swirl_x).clamp(-3.0, 3.0);
						line.vel[j].1 = (line.vel[j].1 * 0.95 + swirl_y).clamp(-3.0, 3.0);
					}
				}
			}
		}
		
		// Spawn new lines when mouse button is held
		if self.mouse_active && self.rng.gen_bool(0.1) {
			if let Some((x, y)) = self.mouse_pos {
				if self.lines.len() < MAX_LINES * 2 {  // Allow more lines when user is active
					let mut new_line = Line::new(&mut self.rng);
					// Start near mouse position
					new_line.pos[0] = (x, y);
					new_line.pos[1] = (
						x + self.rng.gen_range(-new_line.length/2.0..new_line.length/2.0),
						y + self.rng.gen_range(-new_line.length/2.0..new_line.length/2.0)
					);
					self.lines.push(new_line);
				}
			}
		}
		
		// Maintain a reasonable number of lines
		while self.lines.len() > MAX_LINES && !self.mouse_active {
			self.lines.remove(0);
		}
		
		// Maintain line count target (slowly adjust)
		if !self.mouse_active {
			if self.lines.len() < self.target_line_count && self.rng.gen_bool(0.1) {
				self.lines.push(Line::new(&mut self.rng));
			} else if self.lines.len() > self.target_line_count && self.rng.gen_bool(0.1) {
				self.lines.remove(0);
			}
		}
	}
	
	fn draw(&self, frame: &mut [u8]) {
		// Check if we need trails effect (for Rainbow mode)
		let use_trails = self.mode == VisualMode::Rainbow;
		
		if use_trails {
			// Fade the previous frame instead of clearing it
			frame.chunks_exact_mut(4).for_each(|pixel| {
				// Fade each color channel
				pixel[0] = (pixel[0] as f32 * 0.85) as u8;
				pixel[1] = (pixel[1] as f32 * 0.85) as u8;
				pixel[2] = (pixel[2] as f32 * 0.85) as u8;
				pixel[3] = 255; // A
			});
		} else {
			// Set background with custom color
			frame.chunks_exact_mut(4).for_each(|pixel| {
				pixel[0] = self.background_color[0]; // R
				pixel[1] = self.background_color[1]; // G
				pixel[2] = self.background_color[2]; // B
				pixel[3] = 255; // A
			});
		}
		
		// Draw all lines
		for line in &self.lines {
			draw_line(
				frame, 
				line.pos[0].0 as i32, line.pos[0].1 as i32, 
				line.pos[1].0 as i32, line.pos[1].1 as i32, 
				[line.color[0], line.color[1], line.color[2], 255], 
				line.width as i32
			);
		}
		
		// Draw all particles
		for particle in &self.particles {
			// Calculate alpha based on remaining life
			let alpha = ((particle.life / 1.5) * 255.0) as u8;
			
			// Draw the particle as a point with glow
			draw_point(
				frame,
				particle.pos.0 as i32,
				particle.pos.1 as i32,
				[particle.color[0], particle.color[1], particle.color[2], alpha],
				particle.size as i32
			);
		}
	}
	
	fn set_mouse_pos(&mut self, x: f32, y: f32) {
		self.mouse_pos = Some((x, y));
	}
	
	fn set_mouse_active(&mut self, active: bool) {
		self.mouse_active = active;
	}
	
	fn toggle_mode(&mut self) {
		self.mode = match self.mode {
			VisualMode::Normal => VisualMode::Vortex,
			VisualMode::Vortex => VisualMode::Waves,
			VisualMode::Waves => VisualMode::Rainbow,
			VisualMode::Rainbow => VisualMode::Normal,
		};
	}
	
	fn add_lines(&mut self, count: usize) {
		self.target_line_count = (self.target_line_count + count).min(MAX_LINES * 3);
		
		// Immediately add some lines to reach target
		while self.lines.len() < self.target_line_count && self.lines.len() < MAX_LINES * 3 {
			self.lines.push(Line::new(&mut self.rng));
		}
	}
	
	fn remove_lines(&mut self, count: usize) {
		self.target_line_count = self.target_line_count.saturating_sub(count).max(10);
		
		// Remove lines if we have too many
		while self.lines.len() > self.target_line_count && !self.lines.is_empty() {
			self.lines.remove(0);
		}
	}
	
	// Get current info for the window title
	fn get_status(&self) -> String {
		format!("Mesmerise - Mode: {:?} - Lines: {} - Space: change mode, +/-: lines, E: explosion, 1-4: views, Mouse: interact",
			self.mode, self.lines.len())
	}
}

// Bresenham's line algorithm with thickness and glow effect - Improved with safe pixel operations
fn draw_line(frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], width: i32) {
	let dx = (x1 - x0).abs();
	let dy = (y1 - y0).abs();
	let sx = if x0 < x1 { 1 } else { -1 };
	let sy = if y0 < y1 { 1 } else { -1 };
	let mut err = dx - dy;

	let mut x = x0;
	let mut y = y0;
	
	// Extend glow radius beyond the line width
	let glow_radius = width * 3;
	
	// Calculate height for safe pixel operations
	let height = frame.len() / (4 * WIDTH as usize);
	
	// Early culling - check if the line is completely outside the viewport
	if (x0 < 0 && x1 < 0) || (x0 >= WIDTH as i32 && x1 >= WIDTH as i32) ||
	   (y0 < 0 && y1 < 0) || (y0 >= height as i32 && y1 >= height as i32) {
		return;
	}

	while x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
		// Draw the glowing point with falloff
		for w_y in -glow_radius..=glow_radius {
			for w_x in -glow_radius..=glow_radius {
				let px = x + w_x;
				let py = y + w_y;
				
				// Calculate distance from line center
				let distance_squared = w_x * w_x + w_y * w_y;
				let distance = (distance_squared as f32).sqrt();
				
				// Skip if beyond glow radius
				if distance > glow_radius as f32 {
					continue;
				}
				
				// Calculate intensity based on distance
				// Core of the line is solid, then fades out
				let intensity = if distance <= width as f32 {
					1.0 // Solid core
				} else {
					// Quadratic falloff for a smoother glow effect
					let falloff = 1.0 - (distance - width as f32) / (glow_radius as f32 - width as f32);
					falloff * falloff // Squared for more pronounced falloff
				};
				
				// Use our safe blending function
				blend_pixel_safe(
					frame, 
					px, py, 
					WIDTH, HEIGHT as u32, 
					color, 
					intensity
				);
			}
		}

		if x == x1 && y == y1 { break; }

		let e2 = 2 * err;
		if e2 > -dy { err -= dy; x += sx; }
		if e2 < dx { err += dx; y += sy; }
	}
}

// Helper function to draw a glowing point with improved safety
fn draw_point(frame: &mut [u8], x: i32, y: i32, color: [u8; 4], size: i32) {
	let glow_radius = size * 2;
	
	// Early culling - skip if entirely off-screen
	if x + glow_radius < 0 || x - glow_radius >= WIDTH as i32 || 
	   y + glow_radius < 0 || y - glow_radius >= HEIGHT as i32 {
		return;
	}
	
	for w_y in -glow_radius..=glow_radius {
		for w_x in -glow_radius..=glow_radius {
			// Calculate distance from center - use distance squared when possible
			let distance_squared = w_x * w_x + w_y * w_y;
			let distance = (distance_squared as f32).sqrt();
			
			// Skip if beyond glow radius
			if distance > glow_radius as f32 {
				continue;
			}
			
			// Calculate intensity based on distance
			let intensity = if distance <= size as f32 {
				1.0 // Solid core
			} else {
				// Quadratic falloff for a smoother glow effect
				let falloff = 1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32);
				falloff * falloff // Squared for more pronounced falloff
			};
			
			// Apply the color with calculated intensity and respect alpha
			let alpha_factor = color[3] as f32 / 255.0;
			let r = (intensity * color[0] as f32 * alpha_factor) as u8;
			let g = (intensity * color[1] as f32 * alpha_factor) as u8;
			let b = (intensity * color[2] as f32 * alpha_factor) as u8;
			
			// Use our safe blending function
			blend_pixel_safe(
				frame,
				x + w_x, y + w_y,
				WIDTH, HEIGHT as u32,
				[r, g, b, color[3]],
				1.0
			);
		}
	}
}

// Helper function to draw a filled circle with improved safety
fn draw_circle(frame: &mut [u8], x: i32, y: i32, radius: i32, color: [u8; 4], width: u32) {
	let height = frame.len() / (4 * width as usize);
	
	// Early culling - skip if entirely off-screen
	if x + radius < 0 || x - radius >= width as i32 || 
	   y + radius < 0 || y - radius >= height as i32 {
		return;
	}
	
	// Use squared distance for performance
	let radius_sq = radius * radius;
	
	for dy in -radius..=radius {
		for dx in -radius..=radius {
			if dx*dx + dy*dy <= radius_sq {
				// Use our safe pixel setter
				set_pixel_safe(frame, x + dx, y + dy, width, height as u32, color);
			}
		}
	}
}

// Helper function to safely set pixel with bounds checking
fn set_pixel_safe(frame: &mut [u8], x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        let idx = 4 * (y as usize * width as usize + x as usize);
        if idx + 3 < frame.len() {
            frame[idx] = color[0];     // R
            frame[idx + 1] = color[1]; // G
            frame[idx + 2] = color[2]; // B
            frame[idx + 3] = color[3]; // A
        }
    }
}

// Helper function for additive blending with bounds checking
fn blend_pixel_safe(frame: &mut [u8], x: i32, y: i32, width: u32, height: u32, color: [u8; 4], intensity: f32) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        let idx = 4 * (y as usize * width as usize + x as usize);
        if idx + 3 < frame.len() {
            let r = (intensity * color[0] as f32) as u16;
            let g = (intensity * color[1] as f32) as u16;
            let b = (intensity * color[2] as f32) as u16;
            let a = color[3];
            
            frame[idx] = (frame[idx] as u16 + r).min(255) as u8;
            frame[idx + 1] = (frame[idx + 1] as u16 + g).min(255) as u8;
            frame[idx + 2] = (frame[idx + 2] as u16 + b).min(255) as u8;
            frame[idx + 3] = a;
        }
    }
}

fn main() -> Result<(), Error> {
	run_combined()
}

fn run_combined() -> Result<(), Error> {
	let event_loop = EventLoop::new().unwrap();
	let mut input = WinitInputHelper::new();
	
	let window = Arc::new({
		let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
		WindowBuilder::new()
			.with_title("Mesmerise - Combined Visualizations")
			.with_inner_size(size)
			.with_min_inner_size(size)
			.build(&event_loop)
			.unwrap()
	});
	
	// Get the primary monitor and set its dimensions for scaling
	if let Some(monitor) = window.primary_monitor() {
		ray_pattern::set_monitor_dimensions(&monitor);
	}

	let mut pixels = {
		let window_size = window.inner_size();
		let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
		Pixels::new(WIDTH, HEIGHT, surface_texture)?
	};

	let mut world = World::new();
	let mut active_side = ActiveSide::Full;
	let start_time = Instant::now();
	let mut last_frame = Instant::now();
	let target_frame_time = Duration::from_secs_f32(1.0 / 60.0); // 60 FPS target
	
	// Track fullscreen state
	let mut is_fullscreen = false;
	
	// Set the window title to match the initial state
	window.set_title("Mesmerise - Combined Visualizations");

	// Create a clone of the window for the closure
	let window_clone = Arc::clone(&window);

	event_loop.run(move |event, window_target| {
		// Handle input events
		if input.update(&event) {
			// Exit on escape or close
			if input.key_pressed(KeyCode::Escape) || input.close_requested() {
				window_target.exit();
				return;
			}
			
			// Toggle visual mode with space
			if input.key_pressed(KeyCode::Space) {
				world.toggle_mode();
				window_clone.set_title(&format!("Mesmerise - Combined - {}", world.get_status()));
			}

			// Toggle fullscreen with F11 key
			if input.key_pressed(KeyCode::F11) {
				is_fullscreen = !is_fullscreen;
				window_clone.set_fullscreen(if is_fullscreen {
					Some(winit::window::Fullscreen::Borderless(None))
				} else {
					None
				});
				
				// Update monitor dimensions when toggling fullscreen
				if let Some(monitor) = window_clone.primary_monitor() {
					ray_pattern::set_monitor_dimensions(&monitor);
				}
			}
			
			// Toggle fullscreen with Alt+Enter
			if (input.key_pressed(KeyCode::Enter) || input.key_pressed(KeyCode::NumpadEnter)) && 
			   (input.key_held(KeyCode::AltLeft) || input.key_held(KeyCode::AltRight)) {
				is_fullscreen = !is_fullscreen;
				window_clone.set_fullscreen(if is_fullscreen {
					Some(winit::window::Fullscreen::Borderless(None))
				} else {
					None
				});
			}

			// Update mouse position if it's within the window
			if let Some(mouse_pos) = input.cursor() {
				let window_size = window_clone.inner_size();
				
				// Only apply mouse interaction to the left side (original visualization)
				if mouse_pos.0 < window_size.width as f32 / 2.0 {
					let adjusted_x = mouse_pos.0;
					let scale_x = ORIGINAL_WIDTH as f32 / (window_size.width as f32 / 2.0);
					let scale_y = ORIGINAL_HEIGHT as f32 / window_size.height as f32;
					world.set_mouse_pos(adjusted_x * scale_x, mouse_pos.1 * scale_y);
				} else {
					world.mouse_pos = None;
				}
			}
			
			// Track mouse button for interaction - only on the left side
			if let Some(mouse_pos) = input.cursor() {
				let window_size = window_clone.inner_size();
				if mouse_pos.0 < window_size.width as f32 / 2.0 {
					world.set_mouse_active(input.mouse_held(MouseButton::Left)); // Left mouse button
				} else {
					world.set_mouse_active(false);
				}
			}

			// Resize the window
			if let Some(size) = input.window_resized() {
				let _ = pixels.resize_surface(size.width, size.height);
				
				// Update monitor dimensions when the window is resized
				if let Some(monitor) = window_clone.primary_monitor() {
					ray_pattern::set_monitor_dimensions(&monitor);
				}
			}

			// Add explosion on key press
			if input.key_pressed(KeyCode::KeyE) {
				let center_x = ORIGINAL_WIDTH as f32 / 2.0;
				let center_y = HEIGHT as f32 / 2.0;
				world.create_explosion(center_x, center_y, 200);
			}

			// Create explosion at mouse position when right clicking
			if input.mouse_pressed(MouseButton::Right) {
				if let Some((x, y)) = world.mouse_pos {
					world.create_explosion(x, y, 100);
				}
			}

			// Add/remove lines with + and - keys
			if input.key_pressed(KeyCode::Equal) {
				world.add_lines(10);
				window_clone.set_title(&format!("Mesmerise - Combined - {}", world.get_status()));
			}
			
			if input.key_pressed(KeyCode::Minus) {
				world.remove_lines(10);
				window_clone.set_title(&format!("Mesmerise - Combined - {}", world.get_status()));
			}

			// Add another special key for creating fireworks
			if input.key_pressed(KeyCode::KeyF) {
				// Create multiple explosions for a firework effect
				for _ in 0..5 {
					let x = world.rng.gen_range(0.0..ORIGINAL_WIDTH as f32);
					let y = world.rng.gen_range(0.0..HEIGHT as f32 / 2.0); // Upper half of screen
					world.create_explosion(x, y, 50);
				}
			}
			
			// Toggle which side(s) are active
			if input.key_pressed(KeyCode::Digit1) {
				active_side = ActiveSide::Original; // Original only
				window_clone.set_title("Mesmerise - Original Visualization Only");
			}
			if input.key_pressed(KeyCode::Digit2) {
				active_side = ActiveSide::Circular; // Circular only
				window_clone.set_title("Mesmerise - Circular Visualization Only");
			}
			if input.key_pressed(KeyCode::Digit3) {
				active_side = ActiveSide::Full; // Full screen with all visualizations
				window_clone.set_title("Mesmerise - Combined Visualizations");
			}
			if input.key_pressed(KeyCode::Digit4) {
				active_side = ActiveSide::RayPattern; // Only ray pattern
				window_clone.set_title("Mesmerise - Ray Pattern - WASD/Arrows: Move Balls, Mouse: Teleport - With Sorting Visualizations");
				
				// Make sure we're in fullscreen in ray pattern mode
				if !is_fullscreen {
					is_fullscreen = true;
					window_clone.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
				}
			}

			// Apply force to balls in ray pattern when in RayPattern mode
			if active_side == ActiveSide::RayPattern || active_side == ActiveSide::Full {
				// Yellow ball controls (WASD)
				let force = 0.5;
				if input.key_held(KeyCode::KeyW) {
					ray_pattern::apply_force_yellow(0.0, -force);
				}
				if input.key_held(KeyCode::KeyS) {
					ray_pattern::apply_force_yellow(0.0, force);
				}
				if input.key_held(KeyCode::KeyA) {
					ray_pattern::apply_force_yellow(-force, 0.0);
				}
				if input.key_held(KeyCode::KeyD) {
					ray_pattern::apply_force_yellow(force, 0.0);
				}
				
				// Green ball controls (arrow keys)
				if input.key_held(KeyCode::ArrowUp) {
					ray_pattern::apply_force_green(0.0, -force);
				}
				if input.key_held(KeyCode::ArrowDown) {
					ray_pattern::apply_force_green(0.0, force);
				}
				if input.key_held(KeyCode::ArrowLeft) {
					ray_pattern::apply_force_green(-force, 0.0);
				}
				if input.key_held(KeyCode::ArrowRight) {
					ray_pattern::apply_force_green(force, 0.0);
				}
				
				// Press R to restart sorting visualizers
				if input.key_pressed(KeyCode::KeyR) {
					ray_pattern::restart_sorters();
				}
				
				// Teleport balls to mouse position on click
				if input.mouse_pressed(MouseButton::Left) {
					if let Some(mouse_pos) = input.cursor() {
						let mouse_x = mouse_pos.0;
						let mouse_y = mouse_pos.1;
						
						ray_pattern::teleport_yellow(mouse_x, mouse_y);
					}
				}
				
				if input.mouse_pressed(MouseButton::Right) {
					if let Some(mouse_pos) = input.cursor() {
						let mouse_x = mouse_pos.0;
						let mouse_y = mouse_pos.1;
						
						ray_pattern::teleport_green(mouse_x, mouse_y);
					}
				}
			}
		}

		// Update and render with frame rate limiting
		if last_frame.elapsed() >= target_frame_time {
			world.update();
			
			// Clear the frame
			for pixel in pixels.frame_mut().chunks_exact_mut(4) {
				// Default to black
				pixel[0] = 0; // R
				pixel[1] = 0; // G
				pixel[2] = 0; // B
				pixel[3] = 255; // A
			}
			
			// Draw the appropriate visualization based on active side
			let elapsed = start_time.elapsed().as_secs_f32();
			
			match active_side {
				ActiveSide::Original => {
					// Draw only the original lines visualization
					draw_original(&mut pixels, &world);
				},
				ActiveSide::Circular => {
					// Draw only the circular visualization
					draw_circular(&mut pixels, elapsed);
				},
				ActiveSide::Full => {
					// Draw the full screen with multiple visualizations
					draw_full_screen(&mut pixels, &world, elapsed);
				},
				ActiveSide::RayPattern => {
					// Draw only the ray pattern
					draw_ray_pattern(&mut pixels, elapsed);
				},
			}
			
			// ALWAYS draw the particle fountain no matter what
			let frame = pixels.frame_mut();
			draw_particle_fountain(frame, WIDTH, HEIGHT, ORIGINAL_WIDTH, ORIGINAL_HEIGHT, elapsed);
			
			// Render the current frame
			if let Err(err) = pixels.render() {
				eprintln!("Pixels render error: {err}");
				window_target.exit();
				return;
			}
			
			// Request redraw
			window_clone.request_redraw();
			last_frame = Instant::now();
		}
	}).unwrap();
	
	Ok(())
}

// Draw the original visualization on the left half of the screen
fn draw_original(pixels: &mut Pixels, world: &World) {
	// Create a buffer for the original visualization
	let mut original_buffer = vec![0u8; 4 * ORIGINAL_WIDTH as usize * ORIGINAL_HEIGHT as usize];
	
	// Draw the original visualization to this buffer
	world.draw(&mut original_buffer);
	
	// Copy the original visualization to the left side of the combined frame
	let frame = pixels.frame_mut();
	
	for y in 0..ORIGINAL_HEIGHT as usize {
		for x in 0..ORIGINAL_WIDTH as usize {
			let src_idx = 4 * (y * ORIGINAL_WIDTH as usize + x);
			let dst_idx = 4 * (y * WIDTH as usize + x);
			
			// Only copy if within bounds
			if src_idx + 3 < original_buffer.len() && dst_idx + 3 < frame.len() {
				frame[dst_idx] = original_buffer[src_idx];       // R
				frame[dst_idx + 1] = original_buffer[src_idx + 1]; // G
				frame[dst_idx + 2] = original_buffer[src_idx + 2]; // B
				frame[dst_idx + 3] = original_buffer[src_idx + 3]; // A
			}
		}
	}
}

// Draw the circular visualization on the right half of the screen
fn draw_circular(pixels: &mut Pixels, time: f32) {
	// Get dimensions for the circular visuals (right side of screen)
	let circular_width = mesmerise_circular::WIDTH;
	let circular_height = mesmerise_circular::HEIGHT;
	
	// Create a temporary buffer for the circular visualization
	let mut circular_buffer = vec![0u8; 4 * circular_width as usize * circular_height as usize];
	
	// Draw the circular visualization to this buffer
	mesmerise_circular::draw_frame(&mut circular_buffer, time);
	
	// Copy the circular visualization to the right side of the combined frame
	let frame = pixels.frame_mut();
	let x_offset = ORIGINAL_WIDTH as usize;
	
	for y in 0..circular_height as usize {
		for x in 0..circular_width as usize {
			let src_idx = 4 * (y * circular_width as usize + x);
			let dst_idx = 4 * (y * WIDTH as usize + (x + x_offset));
			
			// Only copy if within bounds
			if src_idx + 3 < circular_buffer.len() && dst_idx + 3 < frame.len() {
				frame[dst_idx] = circular_buffer[src_idx];       // R
				frame[dst_idx + 1] = circular_buffer[src_idx + 1]; // G
				frame[dst_idx + 2] = circular_buffer[src_idx + 2]; // B
				frame[dst_idx + 3] = circular_buffer[src_idx + 3]; // A
			}
		}
	}
}

// Draw the full screen visualization with multiple sections
fn draw_full_screen(pixels: &mut Pixels, world: &World, time: f32) {
	// Clear to black first
	for pixel in pixels.frame_mut().chunks_exact_mut(4) {
		pixel[0] = 0; // R
		pixel[1] = 0; // G
		pixel[2] = 0; // B
		pixel[3] = 255; // A
	}
	
	let frame = pixels.frame_mut();
	
	// Section 1: Original visualization (top-left)
	let mut original_buffer = vec![0u8; 4 * ORIGINAL_WIDTH as usize * ORIGINAL_HEIGHT as usize];
	world.draw(&mut original_buffer);
	
	// Copy to top-left quadrant
	for y in 0..ORIGINAL_HEIGHT as usize {
		for x in 0..ORIGINAL_WIDTH as usize {
			let src_idx = 4 * (y * ORIGINAL_WIDTH as usize + x);
			let dst_idx = 4 * (y * WIDTH as usize + x);
			
			if src_idx + 3 < original_buffer.len() && dst_idx + 3 < frame.len() {
				frame[dst_idx] = original_buffer[src_idx];
				frame[dst_idx + 1] = original_buffer[src_idx + 1];
				frame[dst_idx + 2] = original_buffer[src_idx + 2];
				frame[dst_idx + 3] = original_buffer[src_idx + 3];
			}
		}
	}
	
	// Section 2: Circular visualization (top-right)
	let circular_width = mesmerise_circular::WIDTH;
	let circular_height = mesmerise_circular::HEIGHT;
	let mut circular_buffer = vec![0u8; 4 * circular_width as usize * circular_height as usize];
	mesmerise_circular::draw_frame(&mut circular_buffer, time);
	
	// Copy to top-right quadrant
	let x_offset = ORIGINAL_WIDTH as usize;
	for y in 0..circular_height as usize {
		for x in 0..circular_width as usize {
			let src_idx = 4 * (y * circular_width as usize + x);
			let dst_idx = 4 * (y * WIDTH as usize + (x + x_offset));
			
			if src_idx + 3 < circular_buffer.len() && dst_idx + 3 < frame.len() {
				frame[dst_idx] = circular_buffer[src_idx];
				frame[dst_idx + 1] = circular_buffer[src_idx + 1];
				frame[dst_idx + 2] = circular_buffer[src_idx + 2];
				frame[dst_idx + 3] = circular_buffer[src_idx + 3];
			}
		}
	}
	
	// Section 3: Particle fountain (bottom-left)
	draw_particle_fountain(frame, WIDTH, HEIGHT, ORIGINAL_WIDTH, ORIGINAL_HEIGHT, time);
	
	// Section 4: Ray pattern visualization (bottom-right)
	let ray_width = ORIGINAL_WIDTH;
	let ray_height = ORIGINAL_HEIGHT;
	let y_offset = ORIGINAL_HEIGHT as usize;

	// Draw ray pattern directly to the frame buffer with the correct offsets
	let ray_frame = &mut frame[(y_offset * WIDTH as usize * 4)..];
	let ray_offset_x = x_offset;
	let _ray_offset_y = 0; // Within the ray_frame, we're at the top

	// Adjust the frame buffer and offsets for the ray pattern
	ray_pattern::draw_frame(ray_frame, ray_width, ray_height, time, ray_offset_x, WIDTH);
}

// Draw a particle fountain effect in the bottom-left quadrant
fn draw_particle_fountain(frame: &mut [u8], full_width: u32, full_height: u32, quad_width: u32, quad_height: u32, time: f32) {
	// Use y_offset to position particles in bottom-left quadrant
	let y_offset = quad_height as usize;
	
	// Position fountain in the center of bottom-left quadrant
	let fountain_x = quad_width as f32 / 2.0;
	let fountain_y = y_offset as f32 + (full_height as f32 - y_offset as f32) / 2.0;
	
	// MASSIVELY BRIGHT BASE - impossible to miss
	for radius in 0..40 {
		let color_intensity = 255 - (radius * 5).min(255);
		draw_circle(
			frame,
			fountain_x as i32,
			fountain_y as i32,
			40 - radius,
			[255, color_intensity as u8, 0, 255],
			full_width
		);
	}
	
	// Draw huge label text with blinking effect
	let blink = ((time * 5.0).sin() * 0.5 + 0.5) * 255.0;
	draw_huge_text(
		frame,
		"FOUNTAIN", 
		quad_width as i32 / 2 - 200,
		y_offset as i32 + 50,
		[255, blink as u8, blink as u8, 255],
		full_width
	);
	
	// Increase number of particles significantly
	let particles = 1000;
	
	for i in 0..particles {
		// Calculate particle position based on time and index
		let lifetime = 2.0; 
		let particle_time = (time + i as f32 * 0.01) % lifetime;
		let progress = particle_time / lifetime;
		
		// Wider angle spread
		let angle = std::f32::consts::PI / 2.0 + (i as f32 / particles as f32 - 0.5) * std::f32::consts::PI * 1.5;
		
		// Faster initial speed with stronger gravity
		let speed = 600.0 * (1.0 - progress * 0.3);
		let gravity = 800.0;
		
		// Calculate position with trajectory
		let x = fountain_x + angle.cos() * speed * particle_time;
		let y = fountain_y - angle.sin() * speed * particle_time + 0.5 * gravity * particle_time * particle_time;
		
		// Only draw if within the bottom-left quadrant
		if x >= 0.0 && x < quad_width as f32 && y >= quad_height as f32 && y < full_height as f32 {
			// Calculate fade based on lifetime
			let fade = if progress < 0.1 {
				progress / 0.1
			} else if progress > 0.7 {
				(1.0 - progress) / 0.3
			} else {
				1.0
			};
			
			// Vibrant rainbow colors - pure bright colors
			let hue = (i as f32 / particles as f32 + time * 0.3) % 1.0;
			let color = hsv_to_rgb(hue, 1.0, 1.0);
			
			// MUCH larger particles
			let size = 4 + (10.0 * (1.0 - progress)) as i32;
			
			// Draw extra bright particle
			draw_extra_bright_particle(
				frame,
				x as i32,
				y as i32,
				size,
				[color[0], color[1], color[2], (255.0 * fade) as u8],
				full_width
			);
		}
	}
	
	// Draw solid white border around bottom-left quadrant to make it obvious
	// Use time to create a pulsing border that alternates between white and red
	let pulse = (time * 10.0).sin() > 0.0;
	let border_color = if pulse { [255, 0, 0, 255] } else { [255, 255, 255, 255] };
	draw_border(frame, 0, y_offset as i32, quad_width as i32, (full_height - quad_height) as i32, border_color, full_width);
	
	// Also clear the entire quadrant if we're in pulse phase to make it stand out
	if pulse {
		for y in y_offset..(full_height as usize) {
			for x in 0..(quad_width as usize) {
				let idx = 4 * (y * full_width as usize + x);
				if idx + 3 < frame.len() {
					// Every few pixels, draw a bright spot to ensure visibility
					if (x + y) % 20 == 0 {
						frame[idx] = 255;      // R
						frame[idx + 1] = 255;  // G
						frame[idx + 2] = 0;    // B
						frame[idx + 3] = 255;  // A
					}
				}
			}
		}
	}
}

// Draw a super bright particle that's impossible to miss - improved with safety
fn draw_extra_bright_particle(frame: &mut [u8], x: i32, y: i32, size: i32, color: [u8; 4], width: u32) {
	let glow_radius = size * 3;
	let height = frame.len() / (4 * width as usize);
	
	// Early culling - skip if entirely off-screen
	if x + glow_radius < 0 || x - glow_radius >= width as i32 || 
	   y + glow_radius < 0 || y - glow_radius >= height as i32 {
		return;
	}
	
	for dy in -glow_radius..=glow_radius {
		for dx in -glow_radius..=glow_radius {
			let dist_sq = dx * dx + dy * dy;
			
			// Skip if beyond glow radius squared
			if dist_sq > glow_radius * glow_radius {
				continue;
			}
			
			let distance = (dist_sq as f32).sqrt();
			
			// Enhanced brightness even at edge of glow
			let intensity = if distance <= size as f32 {
				2.0  // Extra bright core (values > 1 will be clamped but help with additive blending)
			} else if distance <= glow_radius as f32 {
				1.5 * (1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32))
			} else {
				0.0
			};
			
			// Apply with maximum brightness
			let alpha_factor = color[3] as f32 / 255.0;
			let r = (intensity * color[0] as f32 * alpha_factor * 3.0).min(255.0) as u8;
			let g = (intensity * color[1] as f32 * alpha_factor * 3.0).min(255.0) as u8;
			let b = (intensity * color[2] as f32 * alpha_factor * 3.0).min(255.0) as u8;
			
			// Use our safe blending function with boosted intensity
			blend_pixel_safe(
				frame,
				x + dx, y + dy,
				width, height as u32,
				[r, g, b, 255],
				1.0
			);
		}
	}
}

// Draw a massive text label - much bigger than the previous one - improved with safety
fn draw_huge_text(frame: &mut [u8], text: &str, x: i32, y: i32, color: [u8; 4], width: u32) {
	let char_width = 30;  // Much bigger character size
	let char_height = 50;
	let stroke_width = 4;
	let height = frame.len() / (4 * width as usize);
	
	// Early return if text would be completely off-screen
	if y + char_height < 0 || y >= height as i32 {
		return;
	}
	
	for (i, _c) in text.chars().enumerate() {
		let cx = x + (i as i32 * char_width);
		
		// Skip if this character is outside frame
		if cx + char_width < 0 || cx >= width as i32 {
			continue;
		}
		
		// Draw a filled rectangle for each character
		for dy in 0..char_height {
			for dx in 0..char_width {
				let is_border = dx < stroke_width || dx >= char_width - stroke_width || 
							   dy < stroke_width || dy >= char_height - stroke_width;
				
				if is_border {
					// Use our safe pixel setter
					set_pixel_safe(frame, cx + dx, y + dy, width, height as u32, color);
				}
			}
		}
	}
}

// Draw a border around a rectangle
fn draw_border(frame: &mut [u8], x: i32, y: i32, width: i32, height: i32, color: [u8; 4], stride: u32) {
	let border_width = 3;
	
	// Top border
	for dy in 0..border_width {
		for dx in 0..width {
			let px = x + dx;
			let py = y + dy;
			
			let idx = 4 * (py as usize * stride as usize + px as usize);
			if idx + 3 < frame.len() {
				frame[idx] = color[0];
				frame[idx + 1] = color[1];
				frame[idx + 2] = color[2];
				frame[idx + 3] = color[3];
			}
		}
	}
	
	// Bottom border
	for dy in 0..border_width {
		for dx in 0..width {
			let px = x + dx;
			let py = y + height - 1 - dy;
			
			let idx = 4 * (py as usize * stride as usize + px as usize);
			if idx + 3 < frame.len() {
				frame[idx] = color[0];
				frame[idx + 1] = color[1];
				frame[idx + 2] = color[2];
				frame[idx + 3] = color[3];
			}
		}
	}
	
	// Left border
	for dx in 0..border_width {
		for dy in 0..height {
			let px = x + dx;
			let py = y + dy;
			
			let idx = 4 * (py as usize * stride as usize + px as usize);
			if idx + 3 < frame.len() {
				frame[idx] = color[0];
				frame[idx + 1] = color[1];
				frame[idx + 2] = color[2];
				frame[idx + 3] = color[3];
			}
		}
	}
	
	// Right border
	for dx in 0..border_width {
		for dy in 0..height {
			let px = x + width - 1 - dx;
			let py = y + dy;
			
			let idx = 4 * (py as usize * stride as usize + px as usize);
			if idx + 3 < frame.len() {
				frame[idx] = color[0];
				frame[idx + 1] = color[1];
				frame[idx + 2] = color[2];
				frame[idx + 3] = color[3];
			}
		}
	}
}

// Draw the ray pattern visualization on the full screen
fn draw_ray_pattern(pixels: &mut Pixels, time: f32) {
	// Clear to black first
	for pixel in pixels.frame_mut().chunks_exact_mut(4) {
		pixel[0] = 0; // R
		pixel[1] = 0; // G
		pixel[2] = 0; // B
		pixel[3] = 255; // A
	}
	
	let frame = pixels.frame_mut();
	
	// Draw the ray pattern directly to the frame buffer using the full width and height
	// Use the actual WIDTH and HEIGHT for ray pattern to fill the entire buffer
	ray_pattern::draw_frame(frame, WIDTH, HEIGHT, time, 0, WIDTH);
}
