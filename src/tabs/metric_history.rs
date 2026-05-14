use std::collections::VecDeque;
use std::time::Duration;

pub(super) struct MetricHistory {
    values: VecDeque<f64>,
    last_raw: i64,
    initialized: bool,
    capacity: usize,
    interval: Duration,
}

impl MetricHistory {
    pub(super) fn new(capacity: usize, interval: Duration) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            last_raw: 0,
            initialized: false,
            capacity,
            interval,
        }
    }

    pub(super) fn push_delta(&mut self, raw: i64) {
        if self.initialized {
            let delta = (raw - self.last_raw).max(0) as f64 / self.interval.as_secs_f64();
            if self.values.len() >= self.capacity {
                self.values.pop_front();
            }
            self.values.push_back(delta);
        }
        self.last_raw = raw;
        self.initialized = true;
    }

    pub(super) fn push_absolute(&mut self, raw: i64) {
        if self.values.len() >= self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(raw as f64);
        self.initialized = true;
    }

    pub(super) fn push_value(&mut self, val: f64) {
        if self.values.len() >= self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(val);
        self.initialized = true;
    }

    pub(super) fn chart_data(&self) -> Vec<(f64, f64)> {
        let offset = self.capacity.saturating_sub(self.values.len());
        self.values
            .iter()
            .enumerate()
            .map(|(i, &v)| ((i + offset) as f64, v))
            .collect()
    }

    pub(super) fn current_value(&self) -> Option<f64> {
        self.values.back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn history(capacity: usize) -> MetricHistory {
        MetricHistory::new(capacity, Duration::from_secs(1))
    }

    // --- push_delta ---

    #[test]
    fn push_delta_first_call_does_not_produce_value() {
        let mut h = history(10);
        h.push_delta(100);
        assert_eq!(h.current_value(), None);
    }

    #[test]
    fn push_delta_second_call_yields_difference() {
        let mut h = history(10);
        h.push_delta(100);
        h.push_delta(110);
        assert_eq!(h.current_value(), Some(10.0));
    }

    #[test]
    fn push_delta_counter_reset_clamps_to_zero() {
        let mut h = history(10);
        h.push_delta(200);
        h.push_delta(50); // raw < last_raw: counter reset
        assert_eq!(h.current_value(), Some(0.0));
    }

    #[test]
    fn push_delta_normalizes_by_interval() {
        let mut h = MetricHistory::new(10, Duration::from_secs(5));
        h.push_delta(0);
        h.push_delta(50); // delta=50, interval=5s → 10.0/s
        assert_eq!(h.current_value(), Some(10.0));
    }

    // --- push_absolute / push_value ---

    #[test]
    fn push_absolute_stores_value_as_f64() {
        let mut h = history(10);
        h.push_absolute(42);
        assert_eq!(h.current_value(), Some(42.0));
    }

    #[test]
    fn push_value_stores_value_directly() {
        let mut h = history(10);
        h.push_value(3.14);
        assert_eq!(h.current_value(), Some(3.14));
    }

    // --- current_value ---

    #[test]
    fn current_value_returns_none_on_empty_buffer() {
        let h = history(10);
        assert_eq!(h.current_value(), None);
    }

    // --- ring buffer capacity ---

    #[test]
    fn capacity_evicts_oldest_entry() {
        let mut h = history(3);
        h.push_absolute(1);
        h.push_absolute(2);
        h.push_absolute(3);
        h.push_absolute(4); // evicts 1
        let values: Vec<f64> = h.chart_data().into_iter().map(|(_, v)| v).collect();
        assert_eq!(values, [2.0, 3.0, 4.0]);
    }

    // --- chart_data ---

    #[test]
    fn chart_data_is_empty_when_no_values() {
        let h = history(5);
        assert!(h.chart_data().is_empty());
    }

    #[test]
    fn chart_data_partial_fill_offsets_x_axis() {
        let mut h = history(5);
        h.push_absolute(10);
        h.push_absolute(20);
        // capacity=5, 2 values → offset = 3
        let data = h.chart_data();
        assert_eq!(data[0].0, 3.0);
        assert_eq!(data[1].0, 4.0);
    }

    #[test]
    fn chart_data_full_buffer_starts_at_zero() {
        let mut h = history(3);
        h.push_absolute(1);
        h.push_absolute(2);
        h.push_absolute(3);
        let xs: Vec<f64> = h.chart_data().into_iter().map(|(x, _)| x).collect();
        assert_eq!(xs, [0.0, 1.0, 2.0]);
    }
}
