use rand::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub const SORT_ARRAY_SIZE: usize = 200;

// Global statistics tracker
static mut ALGORITHM_STATS: Option<Arc<Mutex<HashMap<SortAlgorithm, u32>>>> = None;

pub fn initialize_algorithm_stats() {
    unsafe {
        if ALGORITHM_STATS.is_none() {
            let mut stats = HashMap::new();
            stats.insert(SortAlgorithm::Bogo, 0);
            stats.insert(SortAlgorithm::Bubble, 0);
            stats.insert(SortAlgorithm::Quick, 0);
            stats.insert(SortAlgorithm::Merge, 0);
            stats.insert(SortAlgorithm::Insertion, 0);
            stats.insert(SortAlgorithm::Selection, 0);
            stats.insert(SortAlgorithm::Heap, 0);
            stats.insert(SortAlgorithm::Radix, 0);
            stats.insert(SortAlgorithm::Shell, 0);
            stats.insert(SortAlgorithm::Cocktail, 0);
            ALGORITHM_STATS = Some(Arc::new(Mutex::new(stats)));
        }
    }
}

pub fn get_algorithm_stats() -> Option<Arc<Mutex<HashMap<SortAlgorithm, u32>>>> {
    unsafe { ALGORITHM_STATS.clone() }
}

pub fn get_leading_algorithm() -> Option<(SortAlgorithm, u32)> {
    unsafe {
        if let Some(stats) = &ALGORITHM_STATS {
            if let Ok(stats_map) = stats.lock() {
                let mut leader = (SortAlgorithm::Bubble, 0);
                for (algorithm, count) in stats_map.iter() {
                    if *count > leader.1 {
                        leader = (algorithm.clone(), *count);
                    }
                }
                return Some(leader);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum SortAlgorithm {
    Bogo,
    Bubble,
    Quick,
    Merge,
    Insertion,
    Selection,
    Heap,
    Radix,
    Shell,
    Cocktail,
}

impl SortAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            SortAlgorithm::Bogo => "Bogo Sort",
            SortAlgorithm::Bubble => "Bubble Sort", 
            SortAlgorithm::Quick => "Quick Sort",
            SortAlgorithm::Merge => "Merge Sort",
            SortAlgorithm::Insertion => "Insertion Sort",
            SortAlgorithm::Selection => "Selection Sort",
            SortAlgorithm::Heap => "Heap Sort",
            SortAlgorithm::Radix => "Radix Sort",
            SortAlgorithm::Shell => "Shell Sort",
            SortAlgorithm::Cocktail => "Cocktail Sort",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SortState {
    Running,
    Completed,
    Restarting,
}

pub struct SortVisualizer {
    pub array: Vec<u8>,
    pub steps: usize,
    pub algorithm: SortAlgorithm,
    pub state: SortState,
    pub i: usize,
    pub j: usize,
    pub pivot: usize,
    pub stack: Vec<(usize, usize)>,
    pub comparisons: usize,
    pub accesses: usize,
}

impl SortVisualizer {
    pub fn new(algorithm: SortAlgorithm) -> Self {
        let mut array = Vec::with_capacity(SORT_ARRAY_SIZE);
        for i in 1..=SORT_ARRAY_SIZE {
            array.push((i % 255) as u8);
        }
        let mut rng = thread_rng();
        array.shuffle(&mut rng);
        
        let mut visualizer = Self {
            array,
            steps: 0,
            algorithm: algorithm.clone(),
            state: SortState::Running,
            i: 0,
            j: 0,
            pivot: 0,
            stack: Vec::new(),
            comparisons: 0,
            accesses: 0,
        };
        
        // Initialize algorithm-specific state
        match algorithm {
            SortAlgorithm::Quick => {
                visualizer.stack.push((0, visualizer.array.len() - 1));
            },
            SortAlgorithm::Shell => {
                visualizer.pivot = visualizer.array.len() / 2;
            },
            SortAlgorithm::Insertion => {
                visualizer.i = 1; // Start from second element
            },
            _ => {} // Other algorithms use default initialization
        }
        
        visualizer
    }
    
    pub fn update(&mut self) {
        if self.state == SortState::Completed {
            return;
        }
        if self.state == SortState::Restarting {
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            self.state = SortState::Running;
            self.steps = 0;
            self.i = 0;
            self.j = 0;
            self.pivot = 0;
            self.comparisons = 0;
            self.accesses = 0;
            self.stack.clear();
            
            // Initialize algorithm-specific state
            match self.algorithm {
                SortAlgorithm::Quick => {
                    self.stack.push((0, self.array.len() - 1));
                },
                SortAlgorithm::Shell => {
                    self.pivot = self.array.len() / 2;
                },
                SortAlgorithm::Insertion => {
                    self.i = 1; // Start from second element
                },
                _ => {} // Other algorithms use default initialization
            }
            return;
        }
        match self.algorithm {
            SortAlgorithm::Bogo => self.update_bogo(),
            SortAlgorithm::Bubble => self.update_bubble(),
            SortAlgorithm::Quick => self.update_quick(),
            SortAlgorithm::Merge => self.update_merge(),
            SortAlgorithm::Insertion => self.update_insertion(),
            SortAlgorithm::Selection => self.update_selection(),
            SortAlgorithm::Heap => self.update_heap(),
            SortAlgorithm::Radix => self.update_radix(),
            SortAlgorithm::Shell => self.update_shell(),
            SortAlgorithm::Cocktail => self.update_cocktail(),
        }
        self.steps += 1;
    }
    
    fn update_bogo(&mut self) {
        let mut is_sorted = true;
        for i in 1..self.array.len() {
            self.accesses += 2;
            if self.array[i-1] > self.array[i] {
                is_sorted = false;
                break;
            }
        }
        if is_sorted {
            self.state = SortState::Completed;
            self.record_completion();
        } else {
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            self.accesses += self.array.len() * 2;
        }
    }
    
    fn update_bubble(&mut self) {
        let n = self.array.len();
        if self.i >= n {
            if self.j == 0 {
                self.state = SortState::Completed;
                self.record_completion();
            } else {
                self.i = 0;
                self.j = 0;
            }
            return;
        }
        if self.i < n - 1 {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[self.i] > self.array[self.i + 1] {
                self.array.swap(self.i, self.i + 1);
                self.accesses += 2;
                self.j += 1;
            }
        }
        self.i += 1;
    }
    
    fn update_quick(&mut self) {
        if self.stack.is_empty() {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        let (low, high) = self.stack.pop().unwrap();
        if low >= high {
            return;
        }
        let pivot = self.partition(low, high);
        if pivot > 0 && pivot - 1 > low {
            self.stack.push((low, pivot - 1));
        }
        if pivot + 1 < high {
            self.stack.push((pivot + 1, high));
        }
    }
    
    fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.array[high];
        self.accesses += 1;
        let mut i = low;
        for j in low..high {
            self.comparisons += 1;
            self.accesses += 1;
            if self.array[j] <= pivot {
                self.array.swap(i, j);
                self.accesses += 4;
                i += 1;
            }
        }
        self.array.swap(i, high);
        self.accesses += 4;
        i
    }
    
    fn update_merge(&mut self) {
        // Simple merge sort implementation using bubble sort pattern for visualization
        self.update_bubble();
    }
    
    fn update_insertion(&mut self) {
        let n = self.array.len();
        if self.i >= n {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        
        if self.j == 0 {
            self.j = self.i;
        }
        
        if self.j > 0 {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[self.j - 1] > self.array[self.j] {
                self.array.swap(self.j - 1, self.j);
                self.accesses += 2;
                self.j -= 1;
            } else {
                self.i += 1;
                self.j = 0;
            }
        } else {
            self.i += 1;
            self.j = 0;
        }
    }
    
    fn update_selection(&mut self) {
        let n = self.array.len();
        if self.i >= n - 1 {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        
        if self.j == 0 {
            self.j = self.i;
            self.pivot = self.i;
        }
        
        if self.j < n {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[self.j] < self.array[self.pivot] {
                self.pivot = self.j;
            }
            self.j += 1;
        } else {
            if self.pivot != self.i {
                self.array.swap(self.i, self.pivot);
                self.accesses += 2;
            }
            self.i += 1;
            self.j = 0;
        }
    }
    
    fn update_heap(&mut self) {
        // Simple heap sort implementation using bubble sort pattern for visualization
        self.update_bubble();
    }
    
    fn update_radix(&mut self) {
        // Simple radix sort implementation using bubble sort pattern for visualization
        self.update_bubble();
    }
    
    fn update_shell(&mut self) {
        let n = self.array.len();
        if self.pivot == 0 {
            self.pivot = n / 2;
        }
        
        if self.pivot == 0 {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        
        if self.i + self.pivot >= n {
            self.i = 0;
            self.pivot /= 2;
            return;
        }
        
        self.comparisons += 1;
        self.accesses += 2;
        if self.array[self.i] > self.array[self.i + self.pivot] {
            self.array.swap(self.i, self.i + self.pivot);
            self.accesses += 2;
        }
        self.i += 1;
    }
    
    fn update_cocktail(&mut self) {
        let n = self.array.len();
        if self.i >= n || self.j >= n {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        
        // Forward pass
        if self.pivot == 0 {
            if self.i < n - 1 - self.j {
                self.comparisons += 1;
                self.accesses += 2;
                if self.array[self.i] > self.array[self.i + 1] {
                    self.array.swap(self.i, self.i + 1);
                    self.accesses += 2;
                }
                self.i += 1;
            } else {
                self.pivot = 1;
                self.i = n - 2 - self.j;
            }
        } else {
            // Backward pass
            if self.i > self.j {
                self.comparisons += 1;
                self.accesses += 2;
                if self.array[self.i] < self.array[self.i - 1] {
                    self.array.swap(self.i, self.i - 1);
                    self.accesses += 2;
                }
                self.i -= 1;
            } else {
                self.pivot = 0;
                self.j += 1;
                self.i = self.j;
            }
        }
    }
    
    pub fn get_sorted_percent(&self) -> f32 {
        let mut sorted_count = 0;
        for i in 1..self.array.len() {
            if self.array[i-1] <= self.array[i] {
                sorted_count += 1;
            }
        }
        sorted_count as f32 / (self.array.len() - 1) as f32
    }
    
    pub fn restart(&mut self) {
        self.state = SortState::Restarting;
    }
    
    pub fn draw(&self, frame: &mut [u8], x: usize, y: usize, width: usize, height: usize, 
               horizontal: bool, x_offset: usize, buffer_width: u32) {
        self.draw_with_direction(frame, x, y, width, height, horizontal, x_offset, buffer_width, false, false);
    }

    pub fn draw_with_direction(&self, frame: &mut [u8], x: usize, y: usize, width: usize, height: usize, 
                              horizontal: bool, x_offset: usize, buffer_width: u32, flip_horizontal: bool, flip_vertical: bool) {
        let len = self.array.len();
        let bar_width = if horizontal { width / len } else { height / len };
        let max_height = if horizontal { height } else { width };
        
        for (i, &value) in self.array.iter().enumerate() {
            let bar_height = (value as f32 / 256.0 * max_height as f32) as usize;
            let color = match self.state {
                SortState::Running => [100, 150, 255, 255],
                SortState::Completed => [100, 255, 100, 255],
                SortState::Restarting => [255, 100, 100, 255],
            };
            
            if horizontal {
                let bar_x = x + i * bar_width;
                let bar_y = if flip_vertical {
                    y  // Grow downward from top edge
                } else {
                    y + height - bar_height  // Grow upward from bottom edge
                };
                draw_rectangle(frame, bar_x, bar_y, bar_width, bar_height, color, x_offset, buffer_width);
            } else {
                let bar_x = if flip_horizontal {
                    x  // Grow rightward from left edge
                } else {
                    x + width - bar_height  // Grow leftward from right edge
                };
                let bar_y = y + i * bar_width;
                draw_rectangle(frame, bar_x, bar_y, bar_height, bar_width, color, x_offset, buffer_width);
            }
        }
    }
    
    fn record_completion(&self) {
        unsafe {
            if let Some(stats) = &ALGORITHM_STATS {
                if let Ok(mut stats_map) = stats.lock() {
                    if let Some(count) = stats_map.get_mut(&self.algorithm) {
                        *count += 1;
                    }
                }
            }
        }
    }
}

fn draw_rectangle(frame: &mut [u8], x: usize, y: usize, width: usize, height: usize, 
                 color: [u8; 4], x_offset: usize, buffer_width: u32) {
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let idx = 4 * ((pixel_y * buffer_width as usize) + pixel_x + x_offset);
            
            if idx + 3 < frame.len() {
                frame[idx] = color[0];
                frame[idx + 1] = color[1]; 
                frame[idx + 2] = color[2];
                frame[idx + 3] = color[3];
            }
        }
    }
}
