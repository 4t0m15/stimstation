use ab_glyph::{FontArc, PxScale, point, Glyph};
use ab_glyph::Font;

static FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");

pub fn draw_text_ab_glyph(
    frame: &mut [u8],
    text: &str,
    x: f32,
    y: f32,
    color: [u8; 4],
    width: u32,
) {
    let font = FontArc::try_from_slice(FONT_DATA).expect("Font load failed");
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
                        let alpha = (v * color[3] as f32) as u8;
                        frame[idx] = color[0];
                        frame[idx + 1] = color[1];
                        frame[idx + 2] = color[2];
                        frame[idx + 3] = alpha;
                    }
                }
            });
        }
        let h_advance = font.h_advance_unscaled(glyph_id);
        caret.x += h_advance * scale.x;
    }
}
