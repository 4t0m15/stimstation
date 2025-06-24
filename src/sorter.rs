use rand::prelude::*;

pub const SORT_ARRAY_SIZE: usize = 200;

#[derive(Debug, PartialEq)]
pub enum SortAlgorithm {
    Bogo,
    Bubble,
    Quick,
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
        Self {
            array,
            steps: 0,
            algorithm,
            state: SortState::Running,
            i: 0,
            j: 0,
            pivot: 0,
            stack: Vec::new(),
            comparisons: 0,
            accesses: 0,
        }
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
            self.comparisons = 0;
            self.accesses = 0;
            self.stack.clear();
            if self.algorithm == SortAlgorithm::Quick {
                self.stack.push((0, self.array.len() - 1));
            }
            return;
        }
        match self.algorithm {
            SortAlgorithm::Bogo => self.update_bogo(),
            SortAlgorithm::Bubble => self.update_bubble(),
            SortAlgorithm::Quick => self.update_quick(),
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
}
