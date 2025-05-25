		use eframe::App;
		use egui::*;
		use std::*;
		
		//first time using structs :)
		struct FractalClock{
		time: f64;
		paused: bool,
		debt: usize,
		zoom: f32,
		line_width: f32,
		branch_color: Color32,
		}
		
		//default values of the FractalClock Struct
		impl Default for FractalClock {
			fn default() -> Self { // lol I like that rust has a funny arrow
				Self  {
					time: 0.0,
					paused = false,
					depth = 8,
					zoom = 1,
					line_width: 2.5,
					branch_color: Color32::from_rgb(255, 255, 255),
				}
			}
		}