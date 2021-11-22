#[derive(Clone)]
pub struct Counter(Vec<u64>);
impl Counter {
    pub fn new() -> Self {
        Counter(Vec::with_capacity(1000))
    }

    fn get_mut(&mut self, index: usize) -> &mut u64 {
        if index >= self.0.len() {
            let missing = index - self.0.len() + 1;
            for _ in 0..missing {
                self.0.push(0);
            }
        }
        self.0.get_mut(index).unwrap()
    }

    pub fn add(&mut self, index: usize, value: u64) {
        *self.get_mut(index) += value;
    }

    pub fn increment(&mut self, index: usize) {
        *self.get_mut(index) += 1;
    }

    /// return an iterator, skipping the last element cause it's incomplete (the period is not complete)
    pub fn iter(&self) -> impl Iterator<Item = &u64> {
        self.0.iter().take(self.0.len() - 1)
    }

    pub fn cumulative(&self) -> Counter {
        let mut cum_counter = Counter::new();
        let mut cum = 0;
        for (i, val) in self.iter().enumerate() {
            cum += val;
            cum_counter.add(i, *val);
        }
        cum_counter
    }

    pub fn to_vec(&self) -> Vec<u64> {
        self.0.iter().cloned().collect()
    }

    pub fn perc_1000(&self, under: &Counter) -> Vec<u64> {
        self.iter()
            .map(|e| *e as f64)
            .zip(under.iter().map(|e| *e as f64))
            .map(|(over, under)| ((over / under) * 1000.0) as u64)
            .collect()
    }
}

impl Default for Counter {
    fn default() -> Self {
        Counter::new()
    }
}
