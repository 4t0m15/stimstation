use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use rand::thread_rng;
use std::sync::Arc;
use winit::{
	dpi::LogicalSize,
	event::{MouseButton},
	keyboard::KeyCode,
	event_loop::EventLoop,
	window::{WindowBuilder},
};
use winit_input_helper::WinitInputHelper;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

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
	RayPattern, // Only show the ray pattern
	Pythagoras, // Pythagoras visualization
	FibonacciSpiral, // Fibonacci spiral visualization
	SimpleProof  // Simple proof visualization
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
	fn new(rng: &mut ThreadRng) -> Self {
		let x = rng.gen_range(0.0..WIDTH as f32);
		let y = rng.gen_range(0.0..HEIGHT as f32);
		let speed = rng.gen_range(0.5..2.5);
		let length = rng.gen_range(30.0..120.0);
		
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
			width: rng.gen_range(1.0..3.5),
			length,
			cycle_speed: rng.gen_range(0.2..1.5),
			cycle_offset: rng.gen_range(0.0..10.0),
		}
	}
	
	fn update(&mut self, rng: &mut ThreadRng, time: f32, mouse_pos: Option<(f32, f32)>) {
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
		if rng.gen::<f64>() < 0.02 {
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
	rng: ThreadRng,
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

// Add a struct to track FPS
struct FpsCounter {
	frame_times: VecDeque<Instant>,
	last_update: Instant,
	current_fps: f32,
	update_interval: Duration,
}

impl FpsCounter {
	fn new() -> Self {
		Self {
			frame_times: VecDeque::with_capacity(100),
			last_update: Instant::now(),
			current_fps: 0.0,
			update_interval: Duration::from_millis(500), // Update FPS display every 500ms
		}
	}

	fn update(&mut self) {
		let now = Instant::now();
		self.frame_times.push_back(now);

		// Remove frames older than 1 second
		while !self.frame_times.is_empty() && now.duration_since(*self.frame_times.front().unwrap()).as_secs_f32() > 1.0 {
			self.frame_times.pop_front();
		}

		// Update the FPS calculation every update_interval
		if now.duration_since(self.last_update) >= self.update_interval {
			self.current_fps = self.frame_times.len() as f32;
			self.last_update = now;
		}
	}

	fn fps(&self) -> f32 {
		self.current_fps
	}
}

impl World {
	fn new() -> Self {
		let mut rng = thread_rng();
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
		if self.mouse_active && self.rng.gen::<f64>() < 0.1 {
			if let Some((x, y)) = self.mouse_pos {
				if self.lines.len() < MAX_LINES * 2 {
					let mut new_line = Line::new(&mut self.rng);
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
			if self.lines.len() < self.target_line_count && self.rng.gen::<f64>() < 0.1 {
				self.lines.push(Line::new(&mut self.rng));
			} else if self.lines.len() > self.target_line_count && self.rng.gen::<f64>() < 0.1 {
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
	
	// Initialize FPS counter
	let mut fps_counter = FpsCounter::new();
	
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
			if input.key_pressed(KeyCode::Digit5) {
				active_side = ActiveSide::Pythagoras; // Only Pythagoras visualization
				window_clone.set_title("Mesmerise - Pythagoras Theorem Visualization");
			}
			if input.key_pressed(KeyCode::Digit6) {
				active_side = ActiveSide::FibonacciSpiral; // Only Fibonacci spiral
				window_clone.set_title("Mesmerise - Fibonacci Spiral Visualization");
			}
			if input.key_pressed(KeyCode::Digit7) {
				active_side = ActiveSide::SimpleProof; // Only simple proof
				window_clone.set_title("Mesmerise - Simple Proof Visualization");
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

		// Separate rendering logic from input handling
		match event {
			winit::event::Event::WindowEvent { 
				event: winit::event::WindowEvent::RedrawRequested, 
				.. 
			} => {
				// Only update and render if enough time has passed since the last frame
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
						ActiveSide::Pythagoras => {
							// Draw Pythagoras visualization
							draw_pythagoras(&mut pixels, elapsed);
						},
						ActiveSide::FibonacciSpiral => {
							// Draw Fibonacci spiral
							draw_fibonacci_spiral(&mut pixels, elapsed);
						},
						ActiveSide::SimpleProof => {
							// Draw simple proof visualization
							draw_simple_proof(&mut pixels, elapsed);
						},
					}
					
					// ALWAYS draw the particle fountain no matter what
					let frame = pixels.frame_mut();
					draw_particle_fountain(frame, WIDTH, HEIGHT, ORIGINAL_WIDTH, ORIGINAL_HEIGHT, elapsed);
					
					// Update FPS counter
					fps_counter.update();
					
					// Draw FPS counter in the bottom left
					let fps_text = format!("FPS: {:.1}", fps_counter.fps());
					draw_text(frame, &fps_text, 10, HEIGHT as i32 - 30, [255, 255, 0, 255], WIDTH);
					
					// Render the current frame
					if let Err(err) = pixels.render() {
						eprintln!("Pixels render error: {err}");
						window_target.exit();
						return;
					}
					
					last_frame = Instant::now();
					
					// Schedule next frame
					window_clone.request_redraw();
				}
			},
			winit::event::Event::AboutToWait => {
				// Request redraw when the event loop is about to wait
				window_clone.request_redraw();
			},
			_ => {},
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

// Draw text with a simple font
fn draw_text(frame: &mut [u8], text: &str, x: i32, y: i32, color: [u8; 4], width: u32) {
	let char_width = 8;
	let char_height = 15;
	let height = frame.len() / (4 * width as usize);
	
	// Early return if text would be completely off-screen
	if y + char_height < 0 || y >= height as i32 {
		return;
	}
	
	for (i, c) in text.chars().enumerate() {
		let cx = x + (i as i32 * char_width);
		
		// Skip if this character is outside frame
		if cx + char_width < 0 || cx >= width as i32 {
			continue;
		}
		
		// Draw character (simple 7-segment style)
		match c {
			'F' => {
				draw_segment(frame, cx, y, true, false, true, false, true, false, false, color, width);
			},
			'P' => {
				draw_segment(frame, cx, y, true, false, true, true, true, false, false, color, width);
			},
			'S' => {
				draw_segment(frame, cx, y, true, true, false, true, false, true, true, color, width);
			},
			':' => {
				// Draw two dots
				for dy in 0..2 {
					for dx in 0..2 {
						set_pixel_safe(frame, cx + 3 + dx, y + 4 + dy, width, height as u32, color);
						set_pixel_safe(frame, cx + 3 + dx, y + 10 + dy, width, height as u32, color);
					}
				}
			},
			'.' => {
				// Draw a single dot
				for dy in 0..2 {
					for dx in 0..2 {
						set_pixel_safe(frame, cx + 3 + dx, y + char_height - 3 + dy, width, height as u32, color);
					}
				}
			},
			'0' => {
				draw_segment(frame, cx, y, true, true, true, false, true, true, true, color, width);
			},
			'1' => {
				draw_segment(frame, cx, y, false, false, true, false, false, true, false, color, width);
			},
			'2' => {
				draw_segment(frame, cx, y, true, true, false, true, true, false, true, color, width);
			},
			'3' => {
				draw_segment(frame, cx, y, true, true, true, true, false, false, true, color, width);
			},
			'4' => {
				draw_segment(frame, cx, y, false, false, true, true, false, true, true, color, width);
			},
			'5' => {
				draw_segment(frame, cx, y, true, true, true, true, false, true, false, color, width);
			},
			'6' => {
				draw_segment(frame, cx, y, true, true, true, true, true, true, false, color, width);
			},
			'7' => {
				draw_segment(frame, cx, y, true, false, true, false, false, true, false, color, width);
			},
			'8' => {
				draw_segment(frame, cx, y, true, true, true, true, true, true, true, color, width);
			},
			'9' => {
				draw_segment(frame, cx, y, true, true, true, true, false, true, true, color, width);
			},
			' ' => {},
			_ => {
				// Draw a rectangle for unknown characters
				for dy in 0..char_height {
					for dx in 0..char_width {
						if dx == 0 || dx == char_width - 1 || dy == 0 || dy == char_height - 1 {
							set_pixel_safe(frame, cx + dx, y + dy, width, height as u32, color);
						}
					}
				}
			}
		}
	}
}

// Helper function to draw 7-segment display style characters
// Segments: a=top, b=top-right, c=bottom-right, d=bottom, e=bottom-left, f=top-left, g=middle
fn draw_segment(frame: &mut [u8], x: i32, y: i32, a: bool, b: bool, c: bool, d: bool, e: bool, f: bool, g: bool, color: [u8; 4], width: u32) {
	let height = frame.len() / (4 * width as usize);
	let thickness = 2;
	
	// Segment a (top horizontal)
	if a {
		for dy in 0..thickness {
			for dx in 0..6 {
				set_pixel_safe(frame, x + 1 + dx, y + dy, width, height as u32, color);
			}
		}
	}
	
	// Segment b (top-right vertical)
	if b {
		for dy in 0..7 {
			for dx in 0..thickness {
				set_pixel_safe(frame, x + 6 - dx, y + 1 + dy, width, height as u32, color);
			}
		}
	}
	
	// Segment c (bottom-right vertical)
	if c {
		for dy in 0..7 {
			for dx in 0..thickness {
				set_pixel_safe(frame, x + 6 - dx, y + 8 + dy, width, height as u32, color);
			}
		}
	}
	
	// Segment d (bottom horizontal)
	if d {
		for dy in 0..thickness {
			for dx in 0..6 {
				set_pixel_safe(frame, x + 1 + dx, y + 14 - dy, width, height as u32, color);
			}
		}
	}
	
	// Segment e (bottom-left vertical)
	if e {
		for dy in 0..7 {
			for dx in 0..thickness {
				set_pixel_safe(frame, x + dx, y + 8 + dy, width, height as u32, color);
			}
		}
	}
	
	// Segment f (top-left vertical)
	if f {
		for dy in 0..7 {
			for dx in 0..thickness {
				set_pixel_safe(frame, x + dx, y + 1 + dy, width, height as u32, color);
			}
		}
	}
	
	// Segment g (middle horizontal)
	if g {
		for dy in 0..thickness {
			for dx in 0..6 {
				set_pixel_safe(frame, x + 1 + dx, y + 7 + dy, width, height as u32, color);
			}
		}
	}
}

// Pythagoras visualization - simpler version of the macroquad example
fn draw_pythagoras(pixels: &mut Pixels, elapsed: f32) {
    // Get dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    
    // Then get the frame
    let frame = pixels.frame_mut();
    
    // Clear the frame with a white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Parameters
    let a = 100.0f32;
    let b = 150.0f32;
    let c = (a*a + b*b).sqrt();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let angle = elapsed * 0.5; // Rotation angle
    
    // Draw big square (c × c) - light gray
    let square_color = [200, 200, 200, 255];
    let half_c = (c / 2.0) as i32;
    
    for y in -half_c..half_c {
        for x in -half_c..half_c {
            set_pixel_safe(frame, 
                           center_x as i32 + x, 
                           center_y as i32 + y, 
                           width, height, 
                           square_color);
        }
    }
    
    // Draw four triangles (blue)
    let triangle_color = [0, 0, 255, 255];
    
    for i in 0..4 {
        let theta = angle + i as f32 * std::f32::consts::FRAC_PI_2;
        
        // Triangle vertices
        let p1_x = center_x + theta.cos() * (c / 2.0);
        let p1_y = center_y + theta.sin() * (c / 2.0);
        
        let p2_x = center_x + (theta + (b as f32).to_radians()).cos() * (a / 2.0);
        let p2_y = center_y + (theta + b.to_radians()).sin() * (a / 2.0);
        
        let p3_x = center_x + (theta - (a as f32).to_radians()).cos() * (b / 2.0);
        let p3_y = center_y + (theta - a.to_radians()).sin() * (b / 2.0);
        
        // Draw filled triangle using a simple scanline algorithm
        draw_triangle_filled(
            frame,
            p1_x as i32, p1_y as i32,
            p2_x as i32, p2_y as i32,
            p3_x as i32, p3_y as i32,
            width, height,
            triangle_color
        );
    }
    
    // Draw explanatory text
    let text_color = [0, 0, 0, 255];
    draw_simple_text(frame, "Pythagoras Theorem: a² + b² = c²", 
                   20, 30, 
                   width, height, 
                   text_color);
    
    let a_squared = (a * a).round() as i32;
    let b_squared = (b * b).round() as i32;
    let c_squared = (c * c).round() as i32;
    
    draw_simple_text(frame, &format!("{} + {} = {}", a_squared, b_squared, c_squared), 
                   20, 50, 
                   width, height, 
                   text_color);
}

// Fibonacci spiral visualization
fn draw_fibonacci_spiral(pixels: &mut Pixels, elapsed: f32) {
    // Store dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    // Then get frame
    let frame = pixels.frame_mut();
    
    // Use elapsed time to add subtle animation effect
    let animation_offset = (elapsed * 0.5).sin() * 5.0;
    
    // Clear frame with white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Calculate first few Fibonacci numbers
    let mut fibonacci = vec![1, 1];
    for i in 2..12 {
        fibonacci.push(fibonacci[i-1] + fibonacci[i-2]);
    }
    
    // Colors for each square
    let colors = [
        [255, 0, 0, 255],    // Red
        [0, 255, 0, 255],    // Green
        [0, 0, 255, 255],    // Blue
        [255, 255, 0, 255],  // Yellow
        [255, 0, 255, 255],  // Magenta
        [0, 255, 255, 255],  // Cyan
        [255, 128, 0, 255],  // Orange
        [128, 0, 255, 255],  // Purple
        [0, 128, 0, 255],    // Dark green
        [128, 128, 255, 255],// Light blue
        [128, 64, 0, 255],   // Brown
        [255, 128, 128, 255],// Pink
    ];
    
    let scale_factor = 4.0; // Scale the spiral to fit the window
    let center_x = width as i32 / 2;
    let center_y = height as i32 / 2;
    let offset_x = center_x - (fibonacci[fibonacci.len()-1] as f32 * scale_factor / 2.0) as i32 + animation_offset as i32;
    let offset_y = center_y - (fibonacci[fibonacci.len()-1] as f32 * scale_factor / 2.0) as i32;
    
    // Draw the squares
    let mut x = 0;
    let mut y = 0;
    let mut direction = 0; // 0: right, 1: down, 2: left, 3: up
    
    for (i, &fib) in fibonacci.iter().enumerate() {
        let size = (fib as f32 * scale_factor) as i32;
        let color = colors[i % colors.len()];
        
        // Draw the square
        for sx in 0..size {
            for sy in 0..size {
                let px = offset_x + x + sx;
                let py = offset_y + y + sy;
                
                // Draw border
                if sx == 0 || sx == size - 1 || sy == 0 || sy == size - 1 {
                    set_pixel_safe(frame, px, py, width, height, [0, 0, 0, 255]);
                } else {
                    // Fill with a lighter version of the color
                    set_pixel_safe(frame, px, py, width, height, 
                                  [color[0]/2 + 128, color[1]/2 + 128, color[2]/2 + 128, 255]);
                }
            }
        }
        
        // Draw a quarter circle in each square to form the spiral
        let radius = size;
        let center_spiral_x;
        let center_spiral_y;
        
        match direction {
            0 => { // right
                center_spiral_x = x + size;
                center_spiral_y = y + size;
                
                // Draw arc - bottom right corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 - rad_angle.sin() * radius as f32;
                    let arc_y = center_spiral_y as f32 - rad_angle.cos() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                x += size;
            },
            1 => { // down
                center_spiral_x = x;
                center_spiral_y = y + size;
                
                // Draw arc - bottom left corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 + rad_angle.cos() * radius as f32;
                    let arc_y = center_spiral_y as f32 - rad_angle.sin() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                y += size;
            },
            2 => { // left
                center_spiral_x = x;
                center_spiral_y = y;
                
                // Draw arc - top left corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 + rad_angle.sin() * radius as f32;
                    let arc_y = center_spiral_y as f32 + rad_angle.cos() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                x -= size;
            },
            3 => { // up
                center_spiral_x = x + size;
                center_spiral_y = y;
                
                // Draw arc - top right corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 - rad_angle.cos() * radius as f32;
                    let arc_y = center_spiral_y as f32 + rad_angle.sin() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                y -= size;
            },
            _ => unreachable!(),
        }
        
        // Change direction for next square
        direction = (direction + 1) % 4;
    }
    
    // Draw explanatory text
    let text_color = [0, 0, 0, 255];    draw_simple_text(frame, "Fibonacci Spiral", 
                  20, 30, 
                  width, height, 
                  text_color);
    
    draw_simple_text(frame, &format!("Fibonacci sequence: {:?}", &fibonacci[..10]), 
              20, 50, 
              width, height, 
              text_color);
}

// Simple proof visualization
fn draw_simple_proof(pixels: &mut Pixels, elapsed: f32) {
    // Store dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    // Then get frame
    let frame = pixels.frame_mut();
    
    // Clear frame with white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Visual proof that 1 + 2 + 3 + ... + n = n(n+1)/2
    let n = ((elapsed.sin() * 4.0 + 10.0) as i32).max(5).min(15); // Vary between 5-15
    let sum = n * (n + 1) / 2;
    
    // Draw title
    let text_color = [0, 0, 0, 255];
    draw_simple_text(frame, &format!("Visual proof: 1 + 2 + 3 + ... + {} = {}*({} + 1)/2 = {}", 
                            n, n, n, sum), 
              20, 30, 
              width, height, 
              text_color);
    
    // Draw triangular pattern of dots
    let dot_size = 5;
    let spacing = 15;
    let start_x = (width as i32 / 2) - (n * spacing / 2);
    let start_y = 100;
    
    // Draw the triangular arrangement
    for i in 1..=n {
        for j in 1..=i {
            let x = start_x + (j - 1) * spacing;
            let y = start_y + (i - 1) * spacing;
            
            // Draw a dot (small filled circle)
            for dy in -dot_size..=dot_size {
                for dx in -dot_size..=dot_size {
                    if dx*dx + dy*dy <= dot_size*dot_size {
                        set_pixel_safe(frame, x + dx, y + dy, width, height, [255, 0, 0, 255]);
                    }
                }
            }
        }
        
        // Draw the row sum
        draw_simple_text(frame, &format!("Row {}: {}", i, i), 
                  start_x + n * spacing + 20, 
                  start_y + (i - 1) * spacing, 
                  width, height, 
                  text_color);
    }
    
    // Draw the rectangle proof (n by n+1 rectangle split into two triangles)
    let rect_start_x = start_x;
    let rect_start_y = start_y + (n + 3) * spacing;
    
    draw_simple_text(frame, "Alternative proof: n(n+1)/2 is half of an n × (n+1) rectangle", 
              20, rect_start_y - 30, 
              width, height, 
              text_color);
    
    // Draw the rectangle
    for i in 0..n {
        for j in 0..n+1 {
            let x = rect_start_x + j * spacing;
            let y = rect_start_y + i * spacing;
            
            // Draw a dot (small filled circle)
            for dy in -dot_size..=dot_size {
                for dx in -dot_size..=dot_size {
                    if dx*dx + dy*dy <= dot_size*dot_size {
                        // Different colors for upper and lower triangles
                        let color = if i + j < n {
                            [0, 0, 255, 255]  // Blue for lower triangle
                        } else {
                            [0, 150, 0, 255]  // Green for upper triangle
                        };
                        
                        set_pixel_safe(frame, x + dx, y + dy, width, height, color);
                    }
                }
            }
        }
    }
    
    // Draw the diagonal line separating the triangles
    for i in 0..=n {
        let x = rect_start_x + i * spacing;
        let y = rect_start_y + (n - i) * spacing;
        
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx*dx + dy*dy <= 4 {
                    set_pixel_safe(frame, x + dx, y + dy, width, height, [0, 0, 0, 255]);
                }
            }
        }
    }
    
    // Show the formula
    draw_simple_text(frame, &format!("Rectangle area: {} × {} = {}", n, n+1, n*(n+1)), 
              rect_start_x, rect_start_y + n * spacing + 30, 
              width, height, 
              text_color);
              
    draw_simple_text(frame, &format!("Triangle area (half): {}/{} = {}", n*(n+1), 2, n*(n+1)/2), 
              rect_start_x, rect_start_y + n * spacing + 50, 
              width, height, 
              text_color);
}

// Helper function to draw filled triangles
fn draw_triangle_filled(
    frame: &mut [u8], 
    x1: i32, y1: i32, 
    x2: i32, y2: i32, 
    x3: i32, y3: i32, 
    width: u32, height: u32, 
    color: [u8; 4]
) {
    // Sort vertices by y-coordinate
    let mut vertices = [(x1, y1), (x2, y2), (x3, y3)];
    vertices.sort_by_key(|&(_, y)| y);
    
    let [(x1, y1), (x2, y2), (x3, y3)] = vertices;
    
    // Draw the top half of the triangle
    if y2 > y1 {
        let slope1 = (x2 - x1) as f32 / (y2 - y1) as f32;
        let slope2 = (x3 - x1) as f32 / (y3 - y1) as f32;
        
        for y in y1..=y2 {
            let dy = y - y1;
            let start_x = (x1 as f32 + slope1 * dy as f32) as i32;
            let end_x = (x1 as f32 + slope2 * dy as f32) as i32;
            
            for x in std::cmp::min(start_x, end_x)..=std::cmp::max(start_x, end_x) {
                set_pixel_safe(frame, x, y, width, height, color);
            }
        }
    }
    
    // Draw the bottom half of the triangle
    if y3 > y2 {
        let slope1 = (x3 - x2) as f32 / (y3 - y2) as f32;
        let slope2 = (x3 - x1) as f32 / (y3 - y1) as f32;
        
        for y in y2+1..=y3 {
            let dy1 = y - y2;
            let dy2 = y - y1;
            let start_x = (x2 as f32 + slope1 * dy1 as f32) as i32;
            let end_x = (x1 as f32 + slope2 * dy2 as f32) as i32;
            
            for x in std::cmp::min(start_x, end_x)..=std::cmp::max(start_x, end_x) {
                set_pixel_safe(frame, x, y, width, height, color);
            }
        }
    }
}

// Helper function to draw simple text
fn draw_simple_text(
    frame: &mut [u8],
    text: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: [u8; 4]
) {
    // Simple ASCII character rendering with fixed-width font
    let char_width = 8;
    let char_height = 10;
    
    for (i, c) in text.chars().enumerate() {
        let cx = x + i as i32 * char_width;
        
        // Skip if out of view
        if cx < 0 || cx >= width as i32 || y < 0 || y >= height as i32 {
            continue;
        }
        
        // Draw each character
        match c {
            'A' => {
                for dy in 0..char_height {
                    for dx in 0..char_width {
                        if (dx == 0 || dx == char_width-1) && dy > 0 || // vertical lines
                           dy == 0 && dx > 0 && dx < char_width-1 ||     // top
                           dy == char_height/2 {                         // middle
                            set_pixel_safe(frame, cx + dx, y + dy, width, height, color);
                        }
                    }
                }
            },
            // Add more characters as needed
            _ => {
                // Simple box for unimplemented characters
                for dy in 0..char_height {
                    for dx in 0..char_width {
                        if dy == 0 || dy == char_height-1 || dx == 0 || dx == char_width-1 {
                            set_pixel_safe(frame, cx + dx, y + dy, width, height, color);
                        }
                    }
                }
            }
        }
    }
}
