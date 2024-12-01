use std::time::{Duration, Instant};

struct BenchEvent {
    _start: Instant,
    _stop: Instant,
    duration: Duration,
}

pub struct Bench {
    bench_events: Vec<BenchEvent>,
    present_events: Vec<BenchEvent>,
    update_events: Vec<BenchEvent>,
}
// My Comment
impl Bench {
    pub fn new() -> Self {
        Self {
            bench_events: vec![],
            present_events: vec![],
            update_events: vec![],
        }
    }
    pub fn summary(&self) -> String {
        if self.bench_events.is_empty() {
            return "N/A".into();
        }
        let total_count = self.bench_events.len();

        let times = self.bench_events.iter().map(|b| b.duration);
        let total_active_ms = times.clone().sum::<Duration>().as_secs_f64() * 1000.;

        let average = total_active_ms / total_count as f64;
        let presents_count = self.present_events.len() as f64;
        let average_present = (self
            .present_events
            .iter()
            .map(|b| b.duration)
            .sum::<Duration>()
            .as_secs_f64()
            * 1000.)
            / presents_count;
        let update_count = self.update_events.len() as f64;
        let average_update = (self
            .update_events
            .iter()
            .map(|b| b.duration)
            .sum::<Duration>()
            .as_secs_f64()
            * 1000.)
            / update_count;
        let min = times.clone().min().unwrap().as_secs_f64() * 1000.;
        let max = times.clone().max().unwrap().as_secs_f64() * 1000.;
        format!(
            "events: {total_count}
            average event (ms): {average}
            presents: {presents_count}
            average present (ms): {average_present}
            updates: {update_count}
            average update (ms): {average_update}
            min: {min}
            max: {max}"
        )
    }
    pub fn add_total(&mut self, start: Instant, stop: Instant) {
        self.bench_events.push(BenchEvent {
            _start: start,
            _stop: stop,
            duration: stop - start,
        });
    }
    pub fn add_present(&mut self, start: Instant, stop: Instant) {
        self.present_events.push(BenchEvent {
            _start: start,
            _stop: stop,
            duration: stop - start,
        });
    }

    pub fn add_update(&mut self, start: Instant, stop: Instant) {
        self.update_events.push(BenchEvent {
            _start: start,
            _stop: stop,
            duration: stop - start,
        });
    }
}

impl Default for Bench {
    fn default() -> Self {
        Self::new()
    }
}
