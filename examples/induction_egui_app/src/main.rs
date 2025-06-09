use eframe::{egui, epi};
use plotters::prelude::*;

struct MyApp { n: usize }

impl Default for MyApp {
    fn default() -> Self { Self { n: 5 } }
}

impl epi::App for MyApp {
    fn name(&self) -> &str { "Odd Sum = n²" }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.n, 1..=20).text("n"));

            // Compute odd numbers and cumulative sums
            let odds: Vec<i32> = (0..self.n).map(|k| (2*k + 1) as i32).collect();
            let sums: Vec<i32> = odds.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) }).collect();
            
            // Show the sums
            ui.label(format!("Odd numbers: {:?}", odds));
            ui.label(format!("Sums: {:?}", sums));
            
            if let Some(last_sum) = sums.last() {
                ui.label(format!("Sum of first {} odd numbers = {} = {}²", 
                                 self.n, last_sum, self.n));
            }
            
            // Draw simple visualization using egui
            let height = 200.0;
            let available_width = ui.available_width();
            let bar_width = available_width / (self.n as f32 + 1.0);
            
            let rect = egui::Rect::from_min_size(
                ui.cursor().min, 
                egui::vec2(available_width, height)
            );
            let response = ui.allocate_rect(rect, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            
            let max_sum = self.n * self.n;
            
            // Draw bars for each sum
            for (i, &sum) in sums.iter().enumerate() {
                let x = (i as f32) * bar_width;
                let bar_height = (sum as f32 / max_sum as f32) * height;
                let top_left = rect.min + egui::vec2(x, height - bar_height);
                let bottom_right = top_left + egui::vec2(bar_width * 0.9, bar_height);
                
                let color = egui::Color32::from_rgb(30, 100, 200);
                painter.rect_filled(
                    egui::Rect::from_min_max(top_left, bottom_right),
                    0.0,
                    color
                );
                
                // Add label
                painter.text(
                    bottom_right + egui::vec2(-bar_width/2.0, 15.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", i+1),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE
                );
            }
            
            // Display perfect square
            ui.add_space(height + 20.0);
            ui.label(format!("The sum forms a perfect square of size {}×{}", self.n, self.n));
            
            let square_size = (ui.available_width() * 0.8).min(200.0);
            let rect = egui::Rect::from_min_size(
                ui.cursor().min, 
                egui::vec2(square_size, square_size)
            );
            let response = ui.allocate_rect(rect, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 100, 200)
            );
        });
    }
}

fn main() { 
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(MyApp::default()), options); 
}
