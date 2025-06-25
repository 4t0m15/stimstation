use glyphon::TextRenderer as GlyphonTextRenderer;
pub struct TextProcessor {
    text_renderer: GlyphonTextRenderer,
}
impl TextProcessor {
    pub fn new(text_renderer: GlyphonTextRenderer) -> Self {
        Self { text_renderer }
    }
    pub fn update(&mut self, time: f32, width: u32, height: u32) {}
    pub fn draw(
        &mut self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        x_offset: usize,
        buffer_width: u32,
    ) {
    }
}
