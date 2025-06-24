use crate::algorithms::sorter::{SortVisualizer, SortAlgorithm, SortState};

static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;

pub fn initialize_sorters() {
    unsafe {
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bubble));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bogo));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
    }
}

pub fn draw_sorter_visualizations(frame: &mut [u8], width: u32, height: u32, time: f32, 
                                 scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32) {
    let scale_factor = (scale_x + scale_y) / 2.0;
    let border_thickness = (height as f32 * 0.05 * scale_factor) as usize;
    let side_width = (width as f32 * 0.05 * scale_factor) as usize;
    
    unsafe {
        update_and_draw_sorter(&mut TOP_SORTER, frame, 0, 0, width as usize, border_thickness, 
                              true, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut BOTTOM_SORTER, frame, 0, height as usize - border_thickness, 
                              width as usize, border_thickness, true, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut LEFT_SORTER, frame, 0, border_thickness, side_width, 
                              height as usize - border_thickness * 2, false, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut RIGHT_SORTER, frame, width as usize - side_width, border_thickness, 
                              side_width, height as usize - border_thickness * 2, false, time, x_offset, buffer_width);
    }
}

fn update_and_draw_sorter(sorter: &mut Option<SortVisualizer>, frame: &mut [u8], 
                         x: usize, y: usize, width: usize, height: usize, 
                         horizontal: bool, time: f32, x_offset: usize, buffer_width: u32) {
    if let Some(sorter) = sorter {
        sorter.update();
        if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
            sorter.restart();
        }
        sorter.draw(frame, x, y, width, height, horizontal, x_offset, buffer_width as u32);
    }
}

pub fn restart_sorters() {
    unsafe {
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.restart();
        }
    }
}
