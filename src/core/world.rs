// World state and management
use std::time::Instant;
use rand::prelude::*;
use crate::types::{Line, Particle, Position, Color, VisualMode};

/// World state containing scene objects
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

impl World {
    /// Create a new world
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut lines = Vec::with_capacity(100);
        
        for _ in 0..50 {
            lines.push(Line::new(&mut rng));
        }
        
        Self {
            lines,
            particles: Vec::with_capacity(1000),
            mouse_pos: None,
            mouse_active: false,
            background_color: Color::new(0, 0, 0),
            mode: VisualMode::Normal,
            target_line_count: 50,
            start_time: Instant::now(),
        }
    }
    
    /// Update the world state
    pub fn update(&mut self, dt: f32) {
        let mut rng = rand::thread_rng();
        
        // Update lines
        for line in self.lines.iter_mut() {
            // Update positions based on velocity
            line.pos[0] += line.vel[0] * dt;
            line.pos[1] += line.vel[1] * dt;
            
            // Bounce off walls
            for i in 0..2 {
                if line.pos[i].x < 0.0 {
                    line.pos[i].x = 0.0;
                    line.vel[i].x = line.vel[i].x.abs();
                } else if line.pos[i].x >= crate::types::WIDTH as f32 {
                    line.pos[i].x = crate::types::WIDTH as f32 - 1.0;
                    line.vel[i].x = -line.vel[i].x.abs();
                }
                
                if line.pos[i].y < 0.0 {
                    line.pos[i].y = 0.0;
                    line.vel[i].y = line.vel[i].y.abs();
                } else if line.pos[i].y >= crate::types::HEIGHT as f32 {
                    line.pos[i].y = crate::types::HEIGHT as f32 - 1.0;
                    line.vel[i].y = -line.vel[i].y.abs();
                }
            }
        }
        
        // Add or remove lines to reach target count
        if self.lines.len() < self.target_line_count {
            self.lines.push(Line::new(&mut rng));
        } else if self.lines.len() > self.target_line_count {
            self.lines.pop();
        }
        
        // Update particles
        self.particles.retain_mut(|p| {
            p.life -= dt;
            if p.life > 0.0 {
                p.pos += p.vel * dt;
                
                // Apply simple gravity
                p.vel.y += 0.5 * dt;
                
                // Check bounds
                if p.pos.x < 0.0 || p.pos.x >= crate::types::WIDTH as f32 ||
                   p.pos.y < 0.0 || p.pos.y >= crate::types::HEIGHT as f32 {
                    return false;
                }
                
                true
            } else {
                false
            }
        });
        
        // Emit particles based on mouse position
        if self.mouse_active {
            if let Some(pos) = self.mouse_pos {
                for _ in 0..5 {
                    self.particles.push(Particle::new(pos, &mut rng));
                }
            }
        }
    }
    
    /// Set the mouse position
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_pos = Some(Position::new(x, y));
    }
    
    /// Set the mouse active state
    pub fn set_mouse_active(&mut self, active: bool) {
        self.mouse_active = active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_creation() {
        let world = World::new();
        
        // Check that lines are created
        assert_eq!(world.lines.len(), 50);
        
        // Check that particles start empty
        assert!(world.particles.is_empty());
    }
    
    #[test]
    fn test_mouse_interaction() {
        let mut world = World::new();
        
        // Initially no mouse position
        assert!(world.mouse_pos.is_none());
        
        // Set mouse position
        world.set_mouse_pos(100.0, 100.0);
        assert!(world.mouse_pos.is_some());
        if let Some(pos) = world.mouse_pos {
            assert_eq!(pos.x, 100.0);
            assert_eq!(pos.y, 100.0);
        }
    }
}
