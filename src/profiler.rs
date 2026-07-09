use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub struct Profiler {
    pub sections: HashMap<&'static str, ProfileSection>,
}

#[derive(Debug, Default)]
pub struct ProfileSection {
    pub total: Duration,
    pub frame: Duration,
    pub calls: u64,
}

pub struct ProfileScope<'a> {
    profiler: &'a mut Profiler,
    name: &'static str,
    start: Instant,
}

impl Profiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset per-frame timings.
    pub fn begin_frame(&mut self) {
        for section in self.sections.values_mut() {
            section.frame = Duration::ZERO;
            section.calls = 0;
        }
    }

    /// Create a scoped timer.
    /// Time is recorded when the returned value is dropped.
    pub fn scope(&mut self, name: &'static str) -> ProfileScope<'_> {
        ProfileScope {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }

    pub fn print(&self) {
        println!("-----------------------------");

        for (name, section) in &self.sections {
            println!(
                "{:<20} {:>8.3} ms  calls: {}",
                name,
                section.frame.as_secs_f64() * 1000.0,
                section.calls
            );
        }
    }

    pub fn get(&self, name: &'static str) -> Option<&ProfileSection> {
        self.sections.get(name)
    }

    pub fn reset(&mut self) {
        self.sections.clear();
    }
}

impl Drop for ProfileScope<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();

        let section = self.profiler.sections.entry(self.name).or_default();

        section.frame += elapsed;
        section.total += elapsed;
        section.calls += 1;
    }
}
