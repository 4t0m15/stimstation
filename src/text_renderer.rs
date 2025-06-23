// Text rendering and fragment management module
use ab_glyph::{Font as _, FontArc, Glyph, PxScale};
use font_kit::{source::SystemSource, properties::Properties};
use once_cell::sync::Lazy;
use rand::prelude::*;

pub struct TextFragment {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: [u8; 3],
    pub life: f32,
    pub max_life: f32,
    pub scale: f32,
}

pub struct TextRenderer {
    fragments: Vec<TextFragment>,
    next_fragment_time: f32,
}

static FONT: Lazy<FontArc> = Lazy::new(|| {
    let source = SystemSource::new();
    let handle = source
        .select_best_match(&[font_kit::family_name::FamilyName::Monospace], &Properties::new())
        .expect("Failed to find a monospace font");

    match handle.load() {
        Ok(font) => {
            let font_data = font.copy_font_data().expect("Failed to get font data");
            FontArc::try_from_vec(font_data.to_vec()).expect("Failed to convert font to FontArc")
        },
        Err(_) => panic!("Failed to load font"),
    }
});

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            fragments: Vec::with_capacity(50),
            next_fragment_time: 0.5,
        }
    }

    pub fn update(&mut self, time: f32, width: u32, height: u32) {
        // Update fragment lifetimes
        self.fragments.retain_mut(|f| {
            f.life -= 1.0 / 60.0; // Assuming 60 FPS
            f.life > 0.0
        });

        // Add new text fragments occasionally
        if time >= self.next_fragment_time {
            self.add_random_text_fragment(width, height);
            
            // Schedule next fragment - more frequent appearance (0.2-1.0 seconds)
            let mut rng = rand::thread_rng();
            let delay = rng.gen_range(0.2..1.0);
            self.next_fragment_time = time + delay;
        }
    }

    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32, x_offset: usize, buffer_width: u32) {
        for fragment in self.fragments.iter() {
            self.draw_text_fragment(frame, width, height, fragment, x_offset, buffer_width);
        }
    }

    fn add_random_text_fragment(&mut self, width: u32, height: u32) {
        let mut rng = rand::thread_rng();

        let random_val: f64 = rng.gen::<f64>();
        if random_val < 0.1 {
            if self.fragments.len() > 50 {
                return;
            }

            let _segment_length = rng.gen_range(20..80);
            let text = format!("Text fragment {}", rng.gen_range(1..1000));

            let x = rng.gen_range(0..width as i32);
            let y = rng.gen_range(0..height as i32);

            let scale = rng.gen_range(20.0..40.0);
            let color = hsv_to_rgb(rng.gen_range(0.0..360.0), 0.8, 1.0);
            let lifetime = rng.gen_range(5.0..15.0);

            self.fragments.push(TextFragment {
                text,
                x,
                y,
                color,
                life: lifetime,
                max_life: lifetime,
                scale,
            });
        }
    }

    fn draw_text_fragment(
        &self,
        frame: &mut [u8],
        _width: u32,
        height: u32,
        fragment: &TextFragment,
        x_offset: usize,
        buffer_width: u32,
    ) {
        let alpha = (fragment.life / fragment.max_life * 255.0) as u8;
        let color = [fragment.color[0], fragment.color[1], fragment.color[2], alpha];

        // Use ab_glyph to draw the text
        let scale = PxScale::from(fragment.scale);
        let glyphs = layout_paragraph(&FONT, scale, fragment.text.as_str());

        for glyph in glyphs {
            if let Some(outlined) = FONT.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|px, py, v| {
                    let gx = px as i32 + bounds.min.x as i32;
                    let gy = py as i32 + bounds.min.y as i32;

                    let pixel_x = fragment.x + gx;
                    let pixel_y = fragment.y + gy;

                    let final_alpha = (v * (alpha as f32 / 255.0) * 255.0) as u8;

                    put_pixel_safe(
                        frame,
                        pixel_x,
                        pixel_y,
                        buffer_width,
                        height,
                        [color[0], color[1], color[2], final_alpha],
                        x_offset,
                    );
                });
            }
        }
    }
}

fn layout_paragraph(font: &FontArc, scale: PxScale, text: &str) -> Vec<Glyph> {
    let mut glyphs = Vec::new();
    let mut x = 0.0;
    let mut y = 0.0;
    
    for c in text.chars() {
        if c == '\n' {
            x = 0.0;
            y += scale.y;
            continue;
        }
        
        let glyph_id = font.glyph_id(c);
        let mut positioned = glyph_id.with_scale(scale);
        positioned.position = ab_glyph::point(x, y);
        glyphs.push(positioned);
        x += font.h_advance_unscaled(glyph_id) * scale.x;
    }
    
    glyphs
}

// HSV to RGB conversion helper for colorful visualization
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8
    ]
}

// Pixel rendering function with alpha blending for smooth text
fn put_pixel_safe(
    frame: &mut [u8],
    x: i32,
    y: i32,
    buffer_width: u32,
    buffer_height: u32,
    color: [u8; 4],
    x_offset: usize,
) {
    if x >= 0 && x < buffer_width as i32 && y >= 0 && y < buffer_height as i32 {
        let x_global = x as usize + x_offset;
        let y_global = y as usize;

        let idx = 4 * (y_global * buffer_width as usize + x_global);
        if idx + 3 < frame.len() {
            // Simple alpha blending
            let bg_r = frame[idx] as f32;
            let bg_g = frame[idx + 1] as f32;
            let bg_b = frame[idx + 2] as f32;

            let fg_r = color[0] as f32;
            let fg_g = color[1] as f32;
            let fg_b = color[2] as f32;
            let fg_a = color[3] as f32 / 255.0;

            frame[idx] = ((fg_r * fg_a) + (bg_r * (1.0 - fg_a))) as u8;
            frame[idx + 1] = ((fg_g * fg_a) + (bg_g * (1.0 - fg_a))) as u8;
            frame[idx + 2] = ((fg_b * fg_a) + (bg_b * (1.0 - fg_a))) as u8;
            frame[idx + 3] = 255;
        }
    }
}
