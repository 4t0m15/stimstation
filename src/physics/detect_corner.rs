static mut CORNER_HITS: u32 = 0;

/// Increment the corner hit counter
pub fn increment_corner_hit(x: f32, y: f32, width: u32, height: u32) {
    // corner if x<20 or x>width-20 AND y<20 or y>height-20
    let is_corner = (x < 20.0 || x > width as f32 - 20.0) && (y < 20.0 || y > height as f32 - 20.0);
    if is_corner {
        unsafe {
            CORNER_HITS += 1;
        }
    }
}

/// Reset corner hits counter
pub fn reset_corner_hits() {
    unsafe {
        CORNER_HITS = 0;
    }
}

/// Get the total number of corner hits
pub fn get_corner_hits() -> u32 {
    unsafe { CORNER_HITS }
}
