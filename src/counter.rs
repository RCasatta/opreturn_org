#[derive(Clone)]
pub struct Counter(Vec<u64>);

const DEFAULT_MAX_ELEMENTS: usize = 140;

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

    /// return a vec, skipping the last element cause it's incomplete (the period is not complete)
    pub fn finish(&self) -> Vec<u64> {
        let no_last = self.0[..self.0.len().checked_sub(1).unwrap_or(0)].to_vec();
        merge_until(&no_last, DEFAULT_MAX_ELEMENTS)
    }
}

fn merge(data: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity((data.len() + 1) / 2);
    for el in data.chunks(2) {
        let r = match el.get(1) {
            None => el[0] * 2,
            Some(v) => el[0] + *v,
        };
        result.push(r);
    }
    result
}

pub fn merge_until(data: &[u64], max_elem: usize) -> Vec<u64> {
    if data.len() < max_elem {
        data.to_vec()
    } else {
        merge_until(&merge(data), max_elem)
    }
}

pub fn perc_1000(over: &[u64], under: &[u64]) -> Vec<u64> {
    over.iter()
        .zip(under.iter())
        .map(|(over, under)| ((*over as f64 / *under as f64) * 1000.0) as u64)
        .collect()
}

pub fn cumulative(data: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(data.len());
    let mut cum = 0;
    for val in data.iter() {
        cum += val;
        result.push(cum);
    }
    result
}

impl Default for Counter {
    fn default() -> Self {
        Counter::new()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_iter() {}
}
