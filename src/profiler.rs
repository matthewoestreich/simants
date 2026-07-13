use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Profiler {
    sections: RefCell<HashMap<&'static str, ProfileSection>>,
    update_interval: Duration,
    last_update: Instant,
}

#[derive(Debug, Default)]
pub struct ProfileSection {
    pub total: Duration,
    pub frame: Duration,
    pub accumulated: Duration,
    pub calls: u64,
    pub accumulated_calls: u64,
}

pub struct ProfileScope<'a> {
    profiler: &'a Profiler,
    name: &'static str,
    start: Instant,
}

impl Profiler {
    pub fn new(interval_seconds: f32) -> Self {
        Self {
            sections: RefCell::new(HashMap::new()),
            update_interval: Duration::from_secs_f32(interval_seconds),
            last_update: Instant::now(),
        }
    }

    pub fn sections(&self) -> Ref<'_, HashMap<&'static str, ProfileSection>> {
        self.sections.borrow()
    }

    /// Reset per-frame timings.
    pub fn begin_frame(&self) {
        for section in self.sections.borrow_mut().values_mut() {
            section.frame = Duration::ZERO;
            section.calls = 0;
        }
    }

    pub fn end_frame(&mut self) {
        let now = Instant::now();

        for section in self.sections.borrow_mut().values_mut() {
            if now.duration_since(self.last_update) >= self.update_interval {
                section.accumulated = section.frame;
                section.accumulated_calls = section.calls;
            }
            section.frame = Duration::ZERO;
            section.calls = 0;
        }

        if now.duration_since(self.last_update) >= self.update_interval {
            self.last_update = now;
        }
    }

    /// Create a scoped timer.
    /// Time is recorded when the returned value is dropped.
    pub fn scope(&self, name: &'static str) -> ProfileScope<'_> {
        ProfileScope {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("-----------------------------");

        let sections = self.sections.borrow();

        for (name, section) in sections.iter() {
            let t = format!(
                "{:<20} {:>8.3} ms  calls: {}",
                name,
                section.accumulated.as_secs_f64() * 1000.0,
                section.accumulated_calls
            );

            println!("{t}");
        }
    }
}

impl Drop for ProfileScope<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();

        let mut sections = self.profiler.sections.borrow_mut();

        let section = sections.entry(self.name).or_default();
        section.frame += elapsed;
        section.total += elapsed;
        section.calls += 1;
    }
}
