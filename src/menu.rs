use pixels::Pixels;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;
use crate::types::{ActiveSide, Color};

const MENU_TIMEOUT: f32 = 5.0; // Time before menu disappears

// Helper functions for drawing in the menu
fn draw_rectangle(frame: &mut [u8], x: i32, y: i32, width: i32, height: i32, color: Color, frame_width: u32) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;
            
            if px >= 0 && px < frame_width as i32 && py >= 0 && py < (frame.len() / (4 * frame_width as usize)) as i32 {
                let idx = 4 * (py as usize * frame_width as usize + px as usize);
                frame[idx] = color.red;
                frame[idx + 1] = color.green;
                frame[idx + 2] = color.blue;
                frame[idx + 3] = 255;
            }
        }
    }
}

// Helper for drawing text in the menu using the project's text rendering
fn draw_menu_text(
    frame: &mut [u8],
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    frame_width: u32
) {
    use crate::text_rendering::draw_text_ab_glyph;
    draw_text_ab_glyph(
        frame, 
        text, 
        x as f32, 
        y as f32, 
        [color.red, color.green, color.blue, 255], 
        frame_width
    );
}

pub struct Menu {
    visible: bool,
    active_index: usize,
    last_interaction_time: f32,
    options: Vec<MenuOption>,
    has_made_selection: bool,
}

struct MenuOption {
    name: String,
    visualization: ActiveSide,
    description: String,
}

impl Menu {
    pub fn new() -> Self {
        let options = vec![
            MenuOption {
                name: "Original Lines".to_string(),
                visualization: ActiveSide::Original,
                description: "Classic line visualization".to_string(),
            },
            MenuOption {
                name: "Circular".to_string(),
                visualization: ActiveSide::Circular,
                description: "Mesmerizing circular patterns".to_string(),
            },
            MenuOption {
                name: "Ray Pattern".to_string(),
                visualization: ActiveSide::RayPattern,
                description: "Light rays pattern visualization".to_string(),
            },
            MenuOption {
                name: "Pythagoras".to_string(),
                visualization: ActiveSide::Pythagoras,
                description: "Animated Pythagorean theorem proof".to_string(),
            },
            MenuOption {
                name: "Fibonacci Spiral".to_string(),
                visualization: ActiveSide::FibonacciSpiral,
                description: "The golden ratio spiral visualization".to_string(),
            },
            MenuOption {
                name: "Simple Proof".to_string(),
                visualization: ActiveSide::SimpleProof,
                description: "Mathematical proof animation".to_string(),
            },
            MenuOption {
                name: "Combined".to_string(),
                visualization: ActiveSide::Combined,
                description: "All visualizations running simultaneously".to_string(),
            },
            MenuOption {
                name: "Full Experience".to_string(),
                visualization: ActiveSide::Full,
                description: "All visualizations running simultaneously".to_string(),
            },
        ];

        Self {
            visible: true,
            active_index: 0,
            last_interaction_time: 0.0,
            options,
            has_made_selection: false,
        }
    }

    pub fn handle_input(&mut self, input: &WinitInputHelper, elapsed_time: f32) {
        if !self.visible {
            // Show menu with ESC
            if input.key_pressed(KeyCode::Escape) {
                self.visible = true;
                self.last_interaction_time = elapsed_time;
            }
            return;
        }
        
        self.last_interaction_time = elapsed_time;
        
        // Handle keyboard navigation
        if input.key_pressed(KeyCode::ArrowDown) {
            self.active_index = (self.active_index + 1) % self.options.len();
            self.last_interaction_time = elapsed_time;
        }
        
        if input.key_pressed(KeyCode::ArrowUp) {
            self.active_index = if self.active_index == 0 {
                self.options.len() - 1
            } else {
                self.active_index - 1
            };
            self.last_interaction_time = elapsed_time;
        }
        
        // Select current option with Enter/Space
        if input.key_pressed(KeyCode::Enter) || input.key_pressed(KeyCode::Space) {
            self.has_made_selection = true;
            self.visible = false;
        }
        
        // Close menu with ESC
        if input.key_pressed(KeyCode::Escape) {
            self.visible = false;
        }
    }

    pub fn update(&mut self, elapsed_time: f32) {
        if self.visible && elapsed_time - self.last_interaction_time > MENU_TIMEOUT {
            self.visible = false;
        }
    }

    pub fn render(&self, pixels: &mut Pixels) {
        if !self.visible {
            return;
        }
        
        let frame = pixels.frame_mut();
        let width = crate::WIDTH;
        let height = crate::HEIGHT;
        
        // Draw semi-transparent background overlay
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let idx = 4 * (y as usize * width as usize + x as usize);
                // Darken the existing pixel
                frame[idx] = (frame[idx] as f32 * 0.3) as u8;
                frame[idx+1] = (frame[idx+1] as f32 * 0.3) as u8;
                frame[idx+2] = (frame[idx+2] as f32 * 0.3) as u8;
            }
        }
        
        // Draw menu title
        let title = "StimStation Visualizations";
        draw_menu_text(
            frame, 
            title, 
            width as i32 / 2 - (title.len() as i32 * 8), 
            height as i32 / 4, 
            Color::new(255, 255, 255),
            width
        );
        
        // Draw menu options
        for (i, option) in self.options.iter().enumerate() {
            let text_color = if i == self.active_index {
                Color::new(255, 255, 0) // Yellow for selected
            } else {
                Color::new(200, 200, 200) // Light gray for unselected
            };
            
            let y_pos = height as i32 / 3 + i as i32 * 30;
            
            // Draw selection rectangle for active item
            if i == self.active_index {
                let rect_width = option.name.len() as i32 * 16 + 20;
                let rect_height = 25;
                let rect_x = width as i32 / 2 - rect_width / 2;
                let rect_y = y_pos - 20;
                draw_rectangle(
                    frame,
                    rect_x,
                    rect_y,
                    rect_width,
                    rect_height,
                    Color::new(40, 40, 120),
                    width
                );
            }
            
            // Draw option name
            draw_menu_text(
                frame,
                &option.name,
                width as i32 / 2 - (option.name.len() as i32 * 8),
                y_pos,
                text_color,
                width
            );
            
            // Draw description for selected option
            if i == self.active_index {
                draw_menu_text(
                    frame,
                    &option.description,
                    width as i32 / 2 - (option.description.len() as i32 * 4),
                    y_pos + 20,
                    Color::new(180, 180, 180),
                    width
                );
            }
        }
        
        // Draw instructions
        let instructions = "Use Arrow Keys to navigate and Enter to select";
        draw_menu_text(
            frame,
            instructions,
            width as i32 / 2 - (instructions.len() as i32 * 4),
            height as i32 * 3/4,
            Color::new(150, 150, 150),
            width
        );
    }
    
    pub fn get_selected_visualization(&self) -> ActiveSide {
        self.options[self.active_index].visualization
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    pub fn has_made_selection(&self) -> bool {
        self.has_made_selection
    }
    
    pub fn reset_selection(&mut self) {
        self.has_made_selection = false;
    }
    
    pub fn toggle_visibility(&mut self, elapsed_time: f32) {
        self.visible = !self.visible;
        if self.visible {
            self.last_interaction_time = elapsed_time;
        }
    }
}
