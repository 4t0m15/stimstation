use crate::types::{World, Line, Particle, Position, Velocity, Color, VisualMode, hsv_to_rgb, WIDTH, HEIGHT, MAX_LINES};
use glam::Vec2;
use rand::prelude::*;
use rayon::prelude::*;
use std::time::Instant;

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