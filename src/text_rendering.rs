use ab_glyph::{FontArc, PxScale, point, Glyph};
use ab_glyph::Font;

// Fallback to a simple bitmap font approach when system font is not available
static FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");

// Simple bitmap font fallback
fn draw_simple_char(frame: &mut [u8], c: char, x: i32, y: i32, color: [u8; 4], width: u32) {
    // Simple 8x12 bitmap font for basic characters
    let patterns = match c.to_ascii_uppercase() {
        'A' => &[
            0b00111000,
            0b01000100,
            0b10000010,
            0b10000010,
            0b11111110,
            0b10000010,
            0b10000010,
            0b10000010,
            0b00000000,
        ],
        'B' => &[
            0b11111100,
            0b10000010,
            0b10000010,
            0b11111100,
            0b10000010,
            0b10000010,
            0b10000010,
            0b11111100,
            0b00000000,
        ],
        'C' => &[
            0b01111100,
            0b10000010,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
        'D' => &[
            0b11111000,
            0b10001100,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10001100,
            0b11111000,
            0b00000000,
        ],
        'E' => &[
            0b11111110,
            0b10000000,
            0b10000000,
            0b11111100,
            0b10000000,
            0b10000000,
            0b10000000,
            0b11111110,
            0b00000000,
        ],
        'F' => &[
            0b11111110,
            0b10000000,
            0b10000000,
            0b11111100,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b00000000,
        ],
        'G' => &[
            0b01111100,
            0b10000010,
            0b10000000,
            0b10011110,
            0b10000010,
            0b10000010,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
        'H' => &[
            0b10000010,
            0b10000010,
            0b10000010,
            0b11111110,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b00000000,
        ],
        'I' => &[
            0b01111100,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b01111100,
            0b00000000,
        ],
        'L' => &[
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b11111110,
            0b00000000,
        ],
        'M' => &[
            0b10000010,
            0b11000110,
            0b10101010,
            0b10010010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b00000000,
        ],
        'N' => &[
            0b10000010,
            0b11000010,
            0b10100010,
            0b10010010,
            0b10001010,
            0b10000110,
            0b10000010,
            0b10000010,
            0b00000000,
        ],
        'O' => &[
            0b01111100,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
        'P' => &[
            0b11111100,
            0b10000010,
            0b10000010,
            0b11111100,
            0b10000000,
            0b10000000,
            0b10000000,
            0b10000000,
            0b00000000,
        ],
        'R' => &[
            0b11111100,
            0b10000010,
            0b10000010,
            0b11111100,
            0b10010000,
            0b10001000,
            0b10000100,
            0b10000010,
            0b00000000,
        ],
        'S' => &[
            0b01111100,
            0b10000010,
            0b10000000,
            0b01111100,
            0b00000010,
            0b00000010,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
        'T' => &[
            0b11111110,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00000000,
        ],
        'U' => &[
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
        'V' => &[
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b01000100,
            0b00101000,
            0b00010000,
            0b00000000,
        ],
        'W' => &[
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10010010,
            0b10101010,
            0b11000110,
            0b10000010,
            0b00000000,
        ],
        'Y' => &[
            0b10000010,
            0b10000010,
            0b01000100,
            0b00101000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00010000,
            0b00000000,
        ],
        ':' => &[
            0b00000000,
            0b00000000,
            0b00011000,
            0b00011000,
            0b00000000,
            0b00011000,
            0b00011000,
            0b00000000,
            0b00000000,
        ],
        ' ' => &[0; 9],
        _ => &[
            0b01111100,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b10000010,
            0b01111100,
            0b00000000,
        ],
    };
    
    let height = frame.len() / (4 * width as usize);
    for (row, &pattern) in patterns.iter().enumerate() {
        for col in 0..8 {
            if pattern & (1 << (7 - col)) != 0 {
                let px = x + col as i32;
                let py = y + row as i32;
                
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let idx = 4 * (py as usize * width as usize + px as usize);
                    if idx + 3 < frame.len() {
                        frame[idx] = color[0];
                        frame[idx + 1] = color[1];
                        frame[idx + 2] = color[2];
                        frame[idx + 3] = color[3];
                    }
                }
            }
        }
    }
}

fn draw_simple_text(frame: &mut [u8], text: &str, x: f32, y: f32, color: [u8; 4], width: u32) {
    let mut current_x = x as i32;
    let current_y = y as i32;
    
    for c in text.chars() {
        draw_simple_char(frame, c, current_x, current_y, color, width);
        current_x += 10; // Character spacing
    }
}

pub fn draw_text_ab_glyph(
    frame: &mut [u8],
    text: &str,
    x: f32,
    y: f32,
    color: [u8; 4],
    width: u32,
) {
    // Try to use the proper font first
    if let Ok(font) = FontArc::try_from_slice(FONT_DATA) {
        let scale = PxScale::from(24.0);
        let mut caret = point(x, y + 24.0); // y is baseline
        let height = frame.len() / (4 * width as usize);
        
        for c in text.chars() {
            let glyph_id = font.glyph_id(c);
            let glyph = Glyph {
                id: glyph_id,
                scale,
                position: caret,
            };
            
            if let Some(outlined) = font.outline_glyph(glyph.clone()) {
                outlined.draw(|gx, gy, v| {
                    let px = caret.x as i32 + gx as i32;
                    let py = (caret.y as i32 - 24) + gy as i32;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let idx = 4 * (py as usize * width as usize + px as usize);
                        if idx + 3 < frame.len() {
                            let alpha = (v * 255.0) as u8;
                            // Use proper alpha blending
                            if alpha > 128 {
                                frame[idx] = color[0];
                                frame[idx + 1] = color[1];
                                frame[idx + 2] = color[2];
                                frame[idx + 3] = 255;
                            }
                        }
                    }
                });
            }
            
            let h_advance = font.h_advance_unscaled(glyph_id);
            caret.x += h_advance * scale.x;
        }
    } else {
        // Fallback to simple bitmap font
        draw_simple_text(frame, text, x, y, color, width);
    }
}

pub fn draw_keyboard_guide(frame: &mut [u8], width: u32) {
    let help_lines = [
        "Keyboard Controls:",
        "1-8: Switch visualizations",
        "ESC: Toggle menu",
        "H: Toggle this help",
        "F/F11: Toggle fullscreen",
        "Space: Toggle mode",
        "+/-: Add/remove lines",
        "E: Center explosion",
        "Right click: Mouse explosion",
    ];
    
    let start_y = 50.0;
    let line_height = 30.0;
    let color = [255, 255, 255, 255]; // White text
    
    for (i, line) in help_lines.iter().enumerate() {
        draw_text_ab_glyph(
            frame,
            line,
            20.0,
            start_y + i as f32 * line_height,
            color,
            width,
        );
    }
}
