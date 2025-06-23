use ab_glyph::{Font, FontArc, PxScale};
use font_kit::source::SystemSource;
use once_cell::sync::Lazy;
use crate::pixel_utils::blend_pixel_safe;
use crate::types::HEIGHT;

static FONT: Lazy<FontArc> = Lazy::new(|| {
    let handle = SystemSource::new()
        .select_best_match(&[font_kit::family_name::FamilyName::SansSerif], &Default::default())
        .unwrap();
    let font_data = handle.load().unwrap().copy_font_data().unwrap();
    FontArc::try_from_vec((*font_data).clone()).unwrap()
});

pub fn draw_text_ab_glyph(frame: &mut [u8], text: &str, x: f32, y: f32, color: [u8; 4], width: u32) {
    // Set font scale - use a consistent size 
    let scale = PxScale::from(24.0);
    let font = &*FONT;
    
    // Start position for rendering
    let cursor_x = x;
    
    // Layout and render entire text as one unit (much faster than per-character)
    let glyphs: Vec<_> = text.chars()
        .scan(cursor_x, |x_pos, c| {
            // Skip control characters
            if c.is_control() && c != '\n' && c != ' ' {
                return Some(None);
            }
            
            let original_x = *x_pos;
            
            if c == ' ' {
                // Fixed advance for space character
                *x_pos += scale.x * 0.35; // Tighten spacing even more
                return Some(None); // No glyph to render for space
            }
            
            if c == '\n' {
                // No rendering for newlines
                return Some(None);
            }
            
            // Get scaled glyph
            let glyph = font.glyph_id(c).with_scale(scale);
            
            // Update position for next glyph - use a tight spacing factor
            let kerning_factor = 0.42; // Adjusted for closer spacing
            *x_pos += font.h_advance_unscaled(glyph.id) * scale.x * kerning_factor;
            
            Some(font.outline_glyph(glyph).map(|g| (g, original_x)))
        })
        .filter_map(|opt| opt)
        .collect();
    
    // Draw all glyphs at their calculated positions
    for (outlined, x_pos) in glyphs {
        let bounds = outlined.px_bounds();
        
        // Draw the glyph pixels
        outlined.draw(|gx, gy, intensity| {
            // Convert from glyph-local coordinates to frame coordinates
            let px = bounds.min.x + gx as f32;
            let py = bounds.min.y + gy as f32;
            
            // Only draw pixels with sufficient coverage
            if intensity > 0.05 {
                blend_pixel_safe(
                    frame,
                    (x_pos + px) as i32,
                    (y + py) as i32,
                    width,
                    HEIGHT,
                    color,
                    intensity,
                );
            }
        });
    }
}

// Helper function to estimate text width for positioning - much more accurate than character count
pub fn estimate_text_width(text: &str) -> f32 {
    let font = &*FONT;
    let scale = PxScale::from(24.0);
    let kerning_factor = 0.42; // Match the same factor used in draw_text_ab_glyph
    
    let mut width = 0.0;
    for c in text.chars() {
        if c.is_control() && c != ' ' {
            continue;
        }
        
        if c == ' ' {
            width += scale.x * 0.35; // Same as in draw_text_ab_glyph
            continue;
        }
        
        let glyph = font.glyph_id(c).with_scale(scale);
        width += font.h_advance_unscaled(glyph.id) * scale.x * kerning_factor;
    }
    
    width
}

pub fn draw_keyboard_guide(frame: &mut [u8], width: u32) {
    let guide_text = [
        "Keyboard Guide:",
        "[1-8] - Change Visualization",
        "[H] - Toggle Help",
        "[F] or [F11] - Toggle Fullscreen",
        "[Space] - Toggle Mode",
        "[Esc] - Show Menu",
        "[=] - Add Lines",
        "[-] - Remove Lines",
        "[E] - Explosion",
        "Right Mouse - Explosion at cursor",
    ];

    // Render each line of text as a complete string
    let mut y = 30.0; // Start position with padding from the top
    let line_height = 25.0; // Consistent line height
    
    // Draw each line in a single call
    for line in guide_text.iter() {
        draw_text_ab_glyph(frame, line, 10.0, y, [255, 255, 255, 255], width);
        y += line_height;
    }
}
