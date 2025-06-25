use rand::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Default size for sorting arrays - controls the number of elements to sort
pub const SORT_ARRAY_SIZE: usize = 200;

/// Global statistics tracker for algorithm completion counts
/// Uses Arc<Mutex<>> for thread-safe access across the application
/// Maps each sorting algorithm to the number of times it has completed successfully
static mut ALGORITHM_STATS: Option<Arc<Mutex<HashMap<SortAlgorithm, u32>>>> = None;

/// Initializes the global algorithm statistics tracker
/// Creates a HashMap with all sorting algorithms initialized to 0 completions
/// This should be called once at application startup
pub fn initialize_algorithm_stats() {
    unsafe {
        if ALGORITHM_STATS.is_none() {
            let mut stats = HashMap::new();
            // Initialize completion count for all algorithms to 0
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

/// Returns a clone of the global algorithm statistics for external access
/// Used by other modules to read algorithm completion counts
pub fn get_algorithm_stats() -> Option<Arc<Mutex<HashMap<SortAlgorithm, u32>>>> {
    unsafe { ALGORITHM_STATS.clone() }
}

/// Finds and returns the algorithm with the highest completion count
/// Returns None if statistics haven't been initialized
/// Used for displaying leaderboard information
pub fn get_leading_algorithm() -> Option<(SortAlgorithm, u32)> {
    unsafe {
        if let Some(stats) = &ALGORITHM_STATS {
            if let Ok(stats_map) = stats.lock() {
                let mut leader = (SortAlgorithm::Bubble, 0);
                // Find algorithm with highest completion count
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

/// Enumeration of all supported sorting algorithms
/// Each variant represents a different sorting algorithm that can be visualized
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum SortAlgorithm {
    Bogo,       // Random shuffle until sorted (extremely inefficient)
    Bubble,     // Simple comparison-based sort
    Quick,      // Divide-and-conquer algorithm
    Merge,      // Stable divide-and-conquer sort
    Insertion,  // Builds sorted array one element at a time
    Selection,  // Finds minimum element and places it at beginning
    Heap,       // Uses binary heap data structure
    Radix,      // Non-comparison sort using digits/bits
    Shell,      // Generalization of insertion sort with gaps
    Cocktail,   // Bidirectional bubble sort
}

impl SortAlgorithm {
    /// Returns the human-readable name of the sorting algorithm
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

/// Represents the current state of a sorting operation
#[derive(Debug, PartialEq)]
pub enum SortState {
    Running,     // Algorithm is actively sorting
    Completed,   // Array is fully sorted
    Restarting,  // About to shuffle and restart
}

/// Main structure that handles sorting visualization and algorithm execution
/// Contains the array being sorted and all state needed for step-by-step execution
pub struct SortVisualizer {
    pub array: Vec<u8>,              // The array being sorted (values 0-255)
    pub steps: usize,                // Number of algorithm steps taken
    pub algorithm: SortAlgorithm,    // Which sorting algorithm to use
    pub state: SortState,            // Current state of the sorting process
    pub i: usize,                    // Primary index for algorithm state
    pub j: usize,                    // Secondary index for some algorithms
    pub pivot: usize,                // Pivot index for quicksort/shell sort gap size
    pub stack: Vec<(usize, usize)>,  // Stack for recursive algorithms like quicksort
    pub comparisons: usize,          // Count of element comparisons made
    pub accesses: usize,             // Count of array accesses made
}

impl SortVisualizer {
    /// Creates a new SortVisualizer with the default array size
    /// Initializes array with values 1-255 (cycling) and shuffles randomly
    pub fn new(algorithm: SortAlgorithm) -> Self {
        let mut array = Vec::with_capacity(SORT_ARRAY_SIZE);
        // Create array with values 1-255, cycling for larger arrays
        for i in 1..=SORT_ARRAY_SIZE {
            array.push((i % 255) as u8);
        }
        // Shuffle the array to create random starting state
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

        // Initialize algorithm-specific state variables
        match algorithm {
            SortAlgorithm::Quick => {
                // Push initial range onto stack for quicksort
                visualizer.stack.push((0, visualizer.array.len() - 1));
            }
            SortAlgorithm::Shell => {
                // Start with gap size of half the array length
                visualizer.pivot = visualizer.array.len() / 2;
            }
            SortAlgorithm::Insertion => {
                visualizer.i = 1; // Start from second element
            }
            _ => {} // Other algorithms use default initialization
        }

        visualizer
    }

    /// Creates a new SortVisualizer with a custom array size
    /// Useful for performance testing with different data sizes
    pub fn new_with_size(algorithm: SortAlgorithm, size: usize) -> Self {
        let mut array = Vec::with_capacity(size);
        // Create array with values 1-255, cycling for larger arrays
        for i in 1..=size {
            array.push((i % 255) as u8);
        }
        // Shuffle the array to create random starting state
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

        // Initialize algorithm-specific state variables
        match algorithm {
            SortAlgorithm::Quick => {
                // Push initial range for quicksort partitioning
                visualizer.stack.push((0, visualizer.array.len() - 1));
            }
            SortAlgorithm::Shell => {
                // Start with gap size of half the array length
                visualizer.pivot = visualizer.array.len() / 2;
            }
            SortAlgorithm::Insertion => {
                visualizer.i = 1; // Start from second element
            }
            _ => {} // Other algorithms use default initialization
        }

        visualizer
    }

    /// Main update method - advances the sorting algorithm by one step
    /// Called repeatedly to animate the sorting process
    pub fn update(&mut self) {
        // Don't update if sorting is already complete
        if self.state == SortState::Completed {
            return;
        }
        
        // Handle restart state by reshuffling and reinitializing
        if self.state == SortState::Restarting {
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            self.state = SortState::Running;
            
            // Reset all tracking variables
            self.steps = 0;
            self.i = 0;
            self.j = 0;
            self.pivot = 0;
            self.comparisons = 0;
            self.accesses = 0;
            self.stack.clear();

            // Re-initialize algorithm-specific state
            match self.algorithm {
                SortAlgorithm::Quick => {
                    self.stack.push((0, self.array.len() - 1));
                }
                SortAlgorithm::Shell => {
                    self.pivot = self.array.len() / 2;
                }
                SortAlgorithm::Insertion => {
                    self.i = 1; // Start from second element
                }
                _ => {} // Other algorithms use default initialization
            }
            return;
        }
        
        // Dispatch to appropriate algorithm update method
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

    /// Bogo Sort implementation - randomly shuffles until sorted
    /// Extremely inefficient but amusing to watch
    fn update_bogo(&mut self) {
        // Check if array is already sorted
        let mut is_sorted = true;
        for i in 1..self.array.len() {
            self.accesses += 2;
            if self.array[i - 1] > self.array[i] {
                is_sorted = false;
                break;
            }
        }
        if is_sorted {
            self.state = SortState::Completed;
            self.record_completion();
        } else {
            // If not sorted, shuffle randomly and try again
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            self.accesses += self.array.len() * 2;
        }
    }

    /// Bubble Sort implementation - compares adjacent elements and swaps if needed
    /// Simple but inefficient O(n²) algorithm
    fn update_bubble(&mut self) {
        let n = self.array.len();
        if self.i >= n - 1 {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }

        let mut swapped_in_pass = false;
        // Bubble largest element to the end of unsorted portion
        for j in 0..(n - 1 - self.i) {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[j] > self.array[j + 1] {
                self.array.swap(j, j + 1);
                self.accesses += 2;
                swapped_in_pass = true;
            }
        }

        self.i += 1;

        // If no swaps occurred, array is sorted
        if !swapped_in_pass {
            self.state = SortState::Completed;
            self.record_completion();
        }
    }

    /// Quick Sort implementation - uses divide-and-conquer with partitioning
    /// Efficient O(n log n) average case algorithm
    fn update_quick(&mut self) {
        if self.stack.is_empty() {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }
        // Pop next range to partition from stack
        let (low, high) = self.stack.pop().unwrap();
        if low >= high {
            return;
        }
        // Partition and get pivot position
        let pivot = self.partition(low, high);
        // Push sub-ranges onto stack for further partitioning
        if pivot > 0 && pivot - 1 > low {
            self.stack.push((low, pivot - 1));
        }
        if pivot + 1 < high {
            self.stack.push((pivot + 1, high));
        }
    }

    /// Partitioning helper for Quick Sort
    /// Places all elements ≤ pivot on left, > pivot on right
    /// Returns the final position of the pivot element
    fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.array[high]; // Use last element as pivot
        self.accesses += 1;
        let mut i = low; // Index of smaller element
        
        for j in low..high {
            self.comparisons += 1;
            self.accesses += 1;
            if self.array[j] <= pivot {
                self.array.swap(i, j);
                self.accesses += 4;
                i += 1;
            }
        }
        // Place pivot in final position
        self.array.swap(i, high);
        self.accesses += 4;
        i
    }

    /// Merge Sort implementation - currently delegates to bubble sort for simplicity
    /// TODO: Implement proper merge sort with temporary arrays
    fn update_merge(&mut self) {
        // Simple merge sort implementation using bubble sort pattern for visualization
        self.update_bubble();
    }

    /// Insertion Sort implementation - builds sorted array one element at a time
    /// Efficient for small arrays or nearly sorted data
    fn update_insertion(&mut self) {
        let n = self.array.len();
        if self.i >= n {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }

        // Insert current element into sorted portion
        let mut j = self.i;
        while j > 0 {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[j - 1] > self.array[j] {
                self.array.swap(j - 1, j);
                self.accesses += 2;
                j -= 1;
            } else {
                break;
            }
        }
        self.i += 1;
    }

    fn update_selection(&mut self) {
        let n = self.array.len();
        if self.i >= n - 1 {
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }

        let mut min_idx = self.i;
        for j in (self.i + 1)..n {
            self.comparisons += 1;
            self.accesses += 2;
            if self.array[j] < self.array[min_idx] {
                min_idx = j;
            }
        }

        if min_idx != self.i {
            self.array.swap(self.i, min_idx);
            self.accesses += 2;
        }
        self.i += 1;
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
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }

        // Gapped insertion sort
        for i in self.pivot..n {
            let temp = self.array[i];
            self.accesses += 1;
            let mut j = i;
            while j >= self.pivot && self.array[j - self.pivot] > temp {
                self.array[j] = self.array[j - self.pivot];
                self.accesses += 2;
                j -= self.pivot;
            }
            self.array[j] = temp;
            self.accesses += 1;
        }

        self.pivot /= 2;
    }

    /// Cocktail Sort implementation - bidirectional bubble sort
    /// Alternates between forward and backward passes
    fn update_cocktail(&mut self) {
        let n = self.array.len();
        if self.i >= n || self.j >= n { 
            self.state = SortState::Completed;
            self.record_completion();
            return;
        }

        // Forward pass (pivot == 0) - bubble largest to right
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
                // Switch to backward pass
                self.pivot = 1;
                self.i = n - 2 - self.j;
            }
        } else {
            // Backward pass (pivot == 1) - bubble smallest to left
            if self.i > self.j {
                self.comparisons += 1;
                self.accesses += 2;
                if self.array[self.i] < self.array[self.i - 1] {
                    self.array.swap(self.i, self.i - 1);
                    self.accesses += 2;
                }
                self.i -= 1;
            } else {
                // Switch back to forward pass, increment bounds
                self.pivot = 0;
                self.j += 1;
                self.i = self.j;
            }
        }
    }

    /// Calculates what percentage of the array is currently in sorted order
    /// Used for progress tracking and visualization
    pub fn get_sorted_percent(&self) -> f32 {
        let mut sorted_count = 0;
        for i in 1..self.array.len() {
            if self.array[i - 1] <= self.array[i] {
                sorted_count += 1;
            }
        }
        sorted_count as f32 / (self.array.len() - 1) as f32
    }

    /// Triggers a restart of the sorting process
    /// Sets state to Restarting, which will be handled in next update() call
    pub fn restart(&mut self) {
        self.state = SortState::Restarting;
    }

    /// Draws the sorting visualization with default orientation (no flipping)
    /// Convenience method that calls draw_with_direction with flip flags set to false
    pub fn draw(
        &self,
        frame: &mut [u8],
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        horizontal: bool,
        x_offset: usize,
        buffer_width: u32,
    ) {
        self.draw_with_direction(
            frame,
            x,
            y,
            width,
            height,
            horizontal,
            x_offset,
            buffer_width,
            false,
            false,
        );
    }

    /// Draws the sorting visualization with configurable orientation
    /// Can flip horizontally or vertically to accommodate different screen edges
    /// horizontal: true for horizontal bars, false for vertical
    /// flip_horizontal: reverses left/right bar growth direction
    /// flip_vertical: reverses up/down bar growth direction
    pub fn draw_with_direction(
        &self,
        frame: &mut [u8],
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        horizontal: bool,
        x_offset: usize,
        buffer_width: u32,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) {
        let len = self.array.len();
        // Calculate bar width based on orientation
        let bar_width = if horizontal {
            width / len
        } else {
            height / len
        };
        let max_height = if horizontal { height } else { width };

        // Draw each array element as a colored bar
        for (i, &value) in self.array.iter().enumerate() {
            // Scale bar height based on element value (0-255 -> 0-max_height)
            let bar_height = (value as f32 / 256.0 * max_height as f32) as usize;
            
            // Color based on current sorting state
            let color = match self.state {
                SortState::Running => [100, 150, 255, 255],     // Blue while sorting
                SortState::Completed => [100, 255, 100, 255],   // Green when complete
                SortState::Restarting => [255, 100, 100, 255],  // Red when restarting
            };

            if horizontal {
                // Horizontal bars (for top/bottom screen edges)
                let bar_x = x + i * bar_width;
                let bar_y = if flip_vertical {
                    y // Grow downward from top edge
                } else {
                    y + height - bar_height // Grow upward from bottom edge
                };
                draw_rectangle(
                    frame,
                    bar_x,
                    bar_y,
                    bar_width,
                    bar_height,
                    color,
                    x_offset,
                    buffer_width,
                );
            } else {
                // Vertical bars (for left/right screen edges)
                let bar_x = if flip_horizontal {
                    x // Grow rightward from left edge
                } else {
                    x + width - bar_height // Grow leftward from right edge
                };
                let bar_y = y + i * bar_width;
                draw_rectangle(
                    frame,
                    bar_x,
                    bar_y,
                    bar_height,
                    bar_width,
                    color,
                    x_offset,
                    buffer_width,
                );
            }
        }
    }

    /// Records completion of this algorithm in global statistics
    /// Increments the completion count for performance tracking
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

/// Helper function to draw a filled rectangle on the frame buffer
/// Used to render individual bars in the sorting visualization
/// 
/// Parameters:
/// - frame: The pixel buffer to draw into (RGBA format)
/// - x, y: Top-left corner coordinates of the rectangle
/// - width, height: Dimensions of the rectangle
/// - color: RGBA color values [R, G, B, A] (0-255 each)
/// - x_offset: Horizontal offset for the drawing area
/// - buffer_width: Width of the entire frame buffer
fn draw_rectangle(
    frame: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: [u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    // Fill rectangle pixel by pixel
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            // Calculate index in RGBA buffer (4 bytes per pixel)
            let idx = 4 * ((pixel_y * buffer_width as usize) + pixel_x + x_offset);

            // Bounds check to prevent buffer overflow
            if idx + 3 < frame.len() {
                frame[idx] = color[0];     // Red
                frame[idx + 1] = color[1]; // Green  
                frame[idx + 2] = color[2]; // Blue
                frame[idx + 3] = color[3]; // Alpha
            }
        }
    }
}
