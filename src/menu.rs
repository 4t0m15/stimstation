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
    // Pass the entire text string to be rendered as a single unit
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

    pub fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        if !self.visible {
            return;
        }
        
        // Draw semi-transparent background overlay - make it darker for better text visibility
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let idx = 4 * (y as usize * width as usize + x as usize);
                // Darken the existing pixel more for better contrast
                frame[idx] = (frame[idx] as f32 * 0.15) as u8;
                frame[idx+1] = (frame[idx+1] as f32 * 0.15) as u8;
                frame[idx+2] = (frame[idx+2] as f32 * 0.15) as u8;
            }
        }
        
        // Draw opaque menu panel background for readability
        let panel_x = (width as i32) / 4;
        let panel_y = (height as i32) / 6 - 20;
        let panel_w = (width as i32) / 2;
        let panel_h = (height as i32) * 2 / 3 + 40;
        draw_rectangle(
            frame,
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::new(0, 0, 0),
            width
        );
        
        // Draw menu title with larger, more visible text
        let title = "STIMSTATION VISUALIZATIONS";
        // Use the text_rendering module's width estimation function
        use crate::text_rendering::estimate_text_width;
        let title_width = estimate_text_width(title) as i32;
        draw_menu_text(
            frame, 
            title, 
            width as i32 / 2 - title_width / 2, // Center the text properly
            height as i32 / 6, 
            Color::new(255, 255, 255),
            width
        );
        
        // Draw menu options
        for (i, option) in self.options.iter().enumerate() {
            let text_color = if i == self.active_index {
                Color::new(255, 255, 0) // Bright yellow for selected
            } else {
                Color::new(220, 220, 220) // Bright gray for unselected
            };
            
            let y_pos = height as i32 / 3 + i as i32 * 35;
            
            // Draw selection rectangle for active item - make it more visible
            if i == self.active_index {
                let option_width = estimate_text_width(&option.name) as i32;
                let rect_width = option_width + 40; // Add padding
                let rect_height = 30;
                let rect_x = width as i32 / 2 - rect_width / 2;
                let rect_y = y_pos - 5;
                draw_rectangle(
                    frame,
                    rect_x,
                    rect_y,
                    rect_width,
                    rect_height,
                    Color::new(60, 60, 150), // Brighter blue selection
                    width
                );
            }
            
            // Draw option name
            let option_width = estimate_text_width(&option.name) as i32;
            draw_menu_text(
                frame,
                &option.name,
                width as i32 / 2 - option_width / 2,
                y_pos,
                text_color,
                width
            );
            
            // Draw description for selected option
            if i == self.active_index {
                let desc_width = estimate_text_width(&option.description) as i32;
                draw_menu_text(
                    frame,
                    &option.description,
                    width as i32 / 2 - desc_width / 2,
                    y_pos + 20,
                    Color::new(200, 200, 200),
                    width
                );
            }
        }
        
        // Draw instructions
        let instructions = "USE ARROW KEYS TO NAVIGATE AND ENTER TO SELECT";
        let instructions_width = estimate_text_width(instructions) as i32;
        draw_menu_text(
            frame,
            instructions,
            width as i32 / 2 - instructions_width / 2,
            height as i32 * 4/5,
            Color::new(180, 180, 180),
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