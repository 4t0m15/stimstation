// fixed_induction_egui.rs - Modified version that demonstrates how to fix the subtraction overflow error
use eframe::egui;

// Example app structure
struct MyApp {
    // App state would go here
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // Initialize state
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Your UI code here
            draw_grid(ui);
        });
    }
}

// Draw a grid with safe y-coordinate calculations
fn draw_grid(ui: &mut egui::Ui) {
    let square_size = 20.0;
    // Draw some squares in a grid
    for i in 0..5 {
        for j in 0..5 {
            let x = 100.0 + i as f32 * (square_size + 5.0);
            let y = 100.0 + j as f32 * (square_size + 5.0);

            let rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(square_size, square_size)
            );
            let response = ui.allocate_rect(rect, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 100, 200)
            );

            // Demo of the fix for your ray_pattern.rs subtraction overflow issue
            // Always use checked_sub or a safe alternative when working with unsigned numbers

            // This approach safely handles coordinates when y is smaller than the offset
            // Instead of: (y - 20) which would panic if y < 20
            let text_y = if y >= 20.0 { y - 20.0 } else { 0.0 };

            // You could also use the Rust standard library's checked_sub method:
            // let text_y = y.checked_sub(20.0).unwrap_or(0.0);

            // The same principle applies to the other subtraction that was causing issues
            let stats_y = if y >= 10.0 { y - 10.0 } else { 0.0 };

            // This would show text safely at the calculated position without risk of overflow
            if ui.is_rect_visible(rect) {
                painter.text(
                    egui::pos2(x, text_y),
                    egui::Align2::LEFT_TOP,
                    format!("Item {},{}", i, j),
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
            }
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(MyApp::default()), options);
}
