use crate::types::{World, Line, Particle, Position, Color, VisualMode, hsv_to_rgb, WIDTH, HEIGHT, MAX_LINES};
use crate::types::{SimpleWorld, SimpleLine, SimpleParticle, SimplePos, SimpleColor, FpsCounter, Buffers};
use crate::viz::circular;
use rand::prelude::*;
use rayon::prelude::*;
use std::time::{Instant, Duration};
use std::collections::VecDeque;

// Constants for simple world
pub const SIMPLE_WIDTH: u32 = 1600;
pub const SIMPLE_HEIGHT: u32 = 800;
pub const SIMPLE_MAX_LINES: usize = 100;
pub const SIMPLE_ORIGINAL_WIDTH: u32 = 800;
pub const SIMPLE_ORIGINAL_HEIGHT: u32 = 400;

// Convert HSV to RGB for simple types
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

impl World {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            lines: (0..MAX_LINES).map(|_| Line::new(&mut rng)).collect(),
            particles: Vec::with_capacity(500),
            mouse_pos: None,
            mouse_active: false,
            background_color: Color::new(0, 0, 0),
            mode: VisualMode::Normal,
            target_line_count: MAX_LINES,
            start_time: Instant::now(),
        }
    }

    pub fn create_explosion(&mut self, pos: Position, count: usize) {
        let mut rng = thread_rng();
        for _ in 0..count {
            self.particles.push(Particle::new(pos, &mut rng));
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        
        // Update particles in parallel
        self.particles.par_iter_mut().for_each(|particle| {
            particle.update(1.0 / 60.0);
        });
        self.particles.retain(|p| p.life > 0.0);
        
        // Update background color
        let bg_hue = (elapsed * 0.02) % 1.0;
        self.background_color = hsv_to_rgb(bg_hue, 0.5, 0.1);
        
        // Update lines based on mode
        match self.mode {
            VisualMode::Normal => self.update_normal(elapsed),
            VisualMode::Vortex => self.update_vortex(elapsed),
            VisualMode::Waves => self.update_waves(elapsed),
            VisualMode::Rainbow => self.update_rainbow(elapsed),
        }
        
        // Spawn new lines when mouse is active
        if self.mouse_active && random::<f64>() < 0.1 {
            if let Some(pos) = self.mouse_pos {
                if self.lines.len() < MAX_LINES * 2 {
                    let mut rng = thread_rng();
                    let mut new_line = Line::new(&mut rng);
                    new_line.pos[0] = pos;
                    new_line.pos[1] = Position::new(
                        pos.x + rng.gen_range(-new_line.length/2.0..new_line.length/2.0),
                        pos.y + rng.gen_range(-new_line.length/2.0..new_line.length/2.0)
                    );
                    self.lines.push(new_line);
                }
            }
        }
        
        // Maintain line count
        while self.lines.len() > MAX_LINES && !self.mouse_active {
            self.lines.remove(0);
        }
        
        if !self.mouse_active {
            if self.lines.len() < self.target_line_count && random::<f64>() < 0.1 {
                let mut rng = thread_rng();
                self.lines.push(Line::new(&mut rng));
            } else if self.lines.len() > self.target_line_count && random::<f64>() < 0.1 {
                self.lines.remove(0);
            }
        }
    }

    fn update_normal(&mut self, elapsed: f32) {
        self.lines.par_iter_mut().for_each(|line| {
            line.update(elapsed, self.mouse_pos);
        });
    }

    fn update_vortex(&mut self, elapsed: f32) {
        let center = self.mouse_pos.unwrap_or(Position::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0));
        
        self.lines.par_iter_mut().for_each(|line| {
            line.update(elapsed, None);
            
            // Add vortex effect
            for i in 0..2 {
                let dx = line.pos[i].x - center.x;
                let dy = line.pos[i].y - center.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist > 5.0 {
                    let force = 300.0 / dist;
                    let vx = -dy / dist * force;
                    let vy = dx / dist * force;
                    
                    line.vel[i].x = (line.vel[i].x * 0.95 + vx * 0.05).clamp(-5.0, 5.0);
                    line.vel[i].y = (line.vel[i].y * 0.95 + vy * 0.05).clamp(-5.0, 5.0);
                }
            }
        });
    }

    fn update_waves(&mut self, elapsed: f32) {
        let time_factor = elapsed * 2.0;
        
        self.lines.par_iter_mut().for_each(|line| {
            line.update(elapsed, None);
            
            // Add wave effect
            for i in 0..2 {
                let wave_x = (line.pos[i].x / 100.0 + time_factor).sin() * 0.2;
                let wave_y = (line.pos[i].y / 80.0 + time_factor * 0.7).cos() * 0.2;
                
                line.vel[i].x = (line.vel[i].x * 0.95 + wave_x).clamp(-3.0, 3.0);
                line.vel[i].y = (line.vel[i].y * 0.95 + wave_y).clamp(-3.0, 3.0);
            }
        });
    }

    fn update_rainbow(&mut self, elapsed: f32) {
        let global_hue = elapsed * 0.3 % 1.0;
        let line_count = self.lines.len() as f32;
        
        self.lines.par_iter_mut().enumerate().for_each(|(i, line)| {
            line.update(elapsed, None);
            
            // Set rainbow color pattern
            let line_offset = (i as f32 / line_count) * 0.5;
            let hue = (global_hue + line_offset) % 1.0;
            line.color = hsv_to_rgb(hue, 0.9, 0.9);
            
            // Add swirling motion
            for j in 0..2 {
                let angle = elapsed * 0.2 + (i as f32 * 0.01);
                let swirl_x = angle.sin() * 0.2;
                let swirl_y = angle.cos() * 0.2;
                
                line.vel[j].x = (line.vel[j].x * 0.95 + swirl_x).clamp(-3.0, 3.0);
                line.vel[j].y = (line.vel[j].y * 0.95 + swirl_y).clamp(-3.0, 3.0);
            }
        });
    }

    pub fn set_mouse_pos(&mut self, pos: Position) {
        self.mouse_pos = Some(pos);
    }

    pub fn set_mouse_active(&mut self, active: bool) {
        self.mouse_active = active;
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            VisualMode::Normal => VisualMode::Vortex,
            VisualMode::Vortex => VisualMode::Waves,
            VisualMode::Waves => VisualMode::Rainbow,
            VisualMode::Rainbow => VisualMode::Normal,
        };
    }

    pub fn add_lines(&mut self, count: usize) {
        self.target_line_count = (self.target_line_count + count).min(MAX_LINES * 3);
        
        while self.lines.len() < self.target_line_count && self.lines.len() < MAX_LINES * 3 {
            let mut rng = thread_rng();
            self.lines.push(Line::new(&mut rng));
        }
    }

    pub fn remove_lines(&mut self, count: usize) {
        self.target_line_count = self.target_line_count.saturating_sub(count).max(10);
        
        while self.lines.len() > self.target_line_count && !self.lines.is_empty() {
            self.lines.remove(0);
        }
    }

    pub fn get_status(&self) -> String {
        format!("Mesmerise - Mode: {:?} - Lines: {} - Space: change mode, +/-: lines, E: explosion, 1-4: views, Mouse: interact",
            self.mode, self.lines.len())
    }

    pub fn draw(&self, frame: &mut [u8]) {
        // Clear frame with background color
        frame.chunks_exact_mut(4).for_each(|pixel| {
            pixel[0] = self.background_color.red;
            pixel[1] = self.background_color.green;
            pixel[2] = self.background_color.blue;
            pixel[3] = 255;
        });
        
        // Draw all lines
        for line in &self.lines {
            crate::pixel_utils::draw_line(
                frame, 
                line.pos[0].x as i32, line.pos[0].y as i32, 
                line.pos[1].x as i32, line.pos[1].y as i32, 
                [line.color.red, line.color.green, line.color.blue, 255], 
                WIDTH as i32
            );
        }
        
        // Draw particles
        for particle in &self.particles {
            let radius = particle.size as i32;
            let alpha = (particle.life * 255.0) as u8;
            
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let px = particle.pos.x as i32 + dx;
                        let py = particle.pos.y as i32 + dy;
                        
                        if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
                            let idx = 4 * (py as usize * WIDTH as usize + px as usize);
                            if idx + 3 < frame.len() {
                                frame[idx] = particle.color.red;
                                frame[idx + 1] = particle.color.green;
                                frame[idx + 2] = particle.color.blue;
                                frame[idx + 3] = alpha;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Line {
    pub fn update(&mut self, time: f32, mouse_pos: Option<Position>) {
        // Color cycling
        let hue = (time * self.cycle_speed + self.cycle_offset) % 1.0;
        self.color = hsv_to_rgb(hue, 0.8, 0.9);
        
        // Mouse attraction
        if let Some(mouse_pos) = mouse_pos {
            for i in 0..2 {
                let dist = (self.pos[i] - mouse_pos).length();
                
                if dist > 5.0 {
                    let force = 40.0 / dist;
                    let direction = (mouse_pos - self.pos[i]).normalize();
                    let attraction = direction * force;
                    
                    self.vel[i] = (self.vel[i] * 0.95 + attraction * 0.05).clamp_length_max(3.0);
                }
            }
        }
        
        // Update positions
        for i in 0..2 {
            self.pos[i] += self.vel[i];
            
            // Bounce off edges with energy loss
            if self.pos[i].x < 0.0 || self.pos[i].x >= WIDTH as f32 {
                self.vel[i].x *= -0.9;
                self.pos[i].x = self.pos[i].x.clamp(0.0, WIDTH as f32 - 1.0);
            }
            if self.pos[i].y < 0.0 || self.pos[i].y >= HEIGHT as f32 {
                self.vel[i].y *= -0.9;
                self.pos[i].y = self.pos[i].y.clamp(0.0, HEIGHT as f32 - 1.0);
            }
        }
        
        // Maintain line length with spring force
        let diff = self.pos[1] - self.pos[0];
        let current_length = diff.length();
        let difference = current_length - self.length;
        
        if current_length > 0.1 {
            let force = difference * 0.01;
            let direction = diff.normalize();
            
            self.vel[0] += direction * force;
            self.vel[1] -= direction * force;
        }
        
        // Random velocity adjustments
        if random::<f64>() < 0.02 {
            let mut rng = thread_rng();
            for i in 0..2 {
                self.vel[i].x = (self.vel[i].x + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
                self.vel[i].y = (self.vel[i].y + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
            }
        }
    }
}

impl Particle {
    pub fn update(&mut self, dt: f32) {
        // Update position
        self.pos += self.vel;
        
        // Add gravity
        self.vel.y += 0.1;
        
        // Reduce life
        self.life -= dt;
    }
}

impl SimpleLine {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let x = rng.gen_range(0.0..SIMPLE_WIDTH as f32);
        let y = rng.gen_range(0.0..SIMPLE_HEIGHT as f32);
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
    
    pub fn update(&mut self, rng: &mut ThreadRng, time: f32, mouse_pos: Option<SimplePos>) {
        // Color cycling based on time
        let hue = (time * self.cycle_speed + self.cycle_offset) % 1.0;
        self.color = simple_hsv_to_rgb(hue, 0.8, 0.9);
        
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
            if self.pos[i].0 < 0.0 || self.pos[i].0 >= SIMPLE_WIDTH as f32 {
                self.vel[i].0 *= -0.9;  // 10% energy loss on bounce
                self.pos[i].0 = self.pos[i].0.clamp(0.0, SIMPLE_WIDTH as f32 - 1.0);
            }
            if self.pos[i].1 < 0.0 || self.pos[i].1 >= SIMPLE_HEIGHT as f32 {
                self.vel[i].1 *= -0.9;  // 10% energy loss on bounce
                self.pos[i].1 = self.pos[i].1.clamp(0.0, SIMPLE_HEIGHT as f32 - 1.0);
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
        let random_val = rand::random::<f64>();
        if random_val < 0.02 {
            for i in 0..2 {
                self.vel[i].0 = (self.vel[i].0 + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
                self.vel[i].1 = (self.vel[i].1 + rng.gen_range(-0.15..0.15)).clamp(-3.0, 3.0);
            }
        }
    }
}

impl SimpleParticle {
    pub fn new(x: f32, y: f32, rng: &mut impl Rng) -> Self {
        let speed = rng.gen_range(1.0..5.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        Self {
            pos: (x, y),
            vel: (angle.cos() * speed, angle.sin() * speed),
            color: simple_hsv_to_rgb(rng.gen_range(0.0..1.0), 0.9, 1.0),
            life: rng.gen_range(0.5..1.5),
            size: rng.gen_range(1.0..3.0),
        }
    }
    
    pub fn update(&mut self, dt: f32) -> bool {
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

impl SimpleWorld {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            lines: (0..SIMPLE_MAX_LINES).map(|_| SimpleLine::new(&mut rng)).collect(),
            rng,
            start_time: Instant::now(),
            mouse_pos: None,
            mouse_active: false,
            background_color: [0, 0, 0],
            mode: VisualMode::Normal,
            particles: Vec::with_capacity(500),
            target_line_count: SIMPLE_MAX_LINES,
        }
    }
    
    pub fn create_explosion(&mut self, x: f32, y: f32, count: usize) {
        for _ in 0..count {
            self.particles.push(SimpleParticle::new(x, y, &mut self.rng));
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        
        // Update particles
        let dt = 1.0 / 60.0; // Assume 60 FPS for physics
        self.particles.retain_mut(|p| p.update(dt));
        
        // Slowly cycle background color
        let bg_hue = (elapsed * 0.02) % 1.0;
        self.background_color = simple_hsv_to_rgb(bg_hue, 0.5, 0.1); // Low value for dark background
        
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
                let center = self.mouse_pos.unwrap_or((SIMPLE_WIDTH as f32 / 2.0, SIMPLE_HEIGHT as f32 / 2.0));
                
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
                    line.color = simple_hsv_to_rgb(hue, 0.9, 0.9);
                    
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
        let mouse_spawn = rand::random::<f64>();
        if self.mouse_active && mouse_spawn < 0.1 {
            if let Some((x, y)) = self.mouse_pos {
                if self.lines.len() < SIMPLE_MAX_LINES * 2 {
                    let mut new_line = SimpleLine::new(&mut self.rng);
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
        while self.lines.len() > SIMPLE_MAX_LINES && !self.mouse_active {
            self.lines.remove(0);
        }
        
        // Maintain line count target (slowly adjust)
        if !self.mouse_active {
            let add_chance = rand::random::<f64>();
            let remove_chance = rand::random::<f64>();
            if self.lines.len() < self.target_line_count && add_chance < 0.1 {
                self.lines.push(SimpleLine::new(&mut self.rng));
            } else if self.lines.len() > self.target_line_count && remove_chance < 0.1 {
                self.lines.remove(0);
            }
        }
    }
    
    pub fn draw(&self, frame: &mut [u8]) {
        // Clear frame with background color (background_color is [u8; 3])
        frame.chunks_exact_mut(4).for_each(|pixel| {
            pixel[0] = self.background_color[0]; // R
            pixel[1] = self.background_color[1]; // G
            pixel[2] = self.background_color[2]; // B
            pixel[3] = 255; // A
        });
        
        // Draw all lines (pos is [(f32, f32); 2], color is [u8; 3])
        for line in &self.lines {
            crate::pixel_utils::draw_line(
                frame, 
                line.pos[0].0 as i32, line.pos[0].1 as i32, 
                line.pos[1].0 as i32, line.pos[1].1 as i32, 
                [line.color[0], line.color[1], line.color[2], 255], 
                WIDTH as i32
            );
        }
        
        // Draw particles (pos is (f32, f32), color is [u8; 3])
        for particle in &self.particles {
            let radius = particle.size as i32;
            let alpha = (particle.life * 255.0) as u8;
            
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let px = particle.pos.0 as i32 + dx;
                        let py = particle.pos.1 as i32 + dy;
                        
                        if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
                            let idx = 4 * (py as usize * WIDTH as usize + px as usize);
                            if idx + 3 < frame.len() {
                                frame[idx] = particle.color[0];
                                frame[idx + 1] = particle.color[1];
                                frame[idx + 2] = particle.color[2];
                                frame[idx + 3] = alpha;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(100),
            last_update: Instant::now(),
            current_fps: 0.0,
            update_interval: Duration::from_millis(500), // Update FPS display every 500ms
        }
    }

    pub fn update(&mut self) {
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

    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}

impl Buffers {
    pub fn new() -> Self {
        let original = vec![0u8; 4 * SIMPLE_ORIGINAL_WIDTH as usize * SIMPLE_ORIGINAL_HEIGHT as usize];
        let circular = vec![0u8; 4 * circular::WIDTH as usize * circular::HEIGHT as usize];
        let full = vec![0u8; 4 * SIMPLE_WIDTH as usize * SIMPLE_HEIGHT as usize];
        Self { original, circular, full }
    }
    
    pub fn clear(&mut self) {
        self.original.fill(0);
        self.circular.fill(0);
        self.full.fill(0);
    }
}