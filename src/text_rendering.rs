use ab_glyph::{point, Font, FontArc, PxScale};
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
    let scale = PxScale::from(24.0);
    let font = &*FONT;
    let mut caret = point(x, y);

    for c in text.chars() {
        let glyph = font.glyph_id(c).with_scale(scale);
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|px, py, v| {
                let px = bounds.min.x + px as f32;
                let py = bounds.min.y + py as f32;
                if v > 0.1 { // Only draw pixels with some coverage
                    blend_pixel_safe(
                        frame,
                        (caret.x + px) as i32,
                        (caret.y + py) as i32,
                        width,
                        HEIGHT,
                        color,
                        v,
                    );
                }
            });
            caret.x += glyph.advance_width();
        }
    }
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

    let mut y = 30.0; // Give some padding from the top
    for line in guide_text.iter() {
        draw_text_ab_glyph(frame, line, 10.0, y, [255, 255, 255, 255], width);
        y += 25.0;
    }
}
