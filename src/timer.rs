use crate::settings::Settings;
use serde::Serialize;
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Timer {
    pub running: bool,
    paused: bool,
    completed: bool,
    work: bool,
    cycle: u32,
    total: u32,
    work_duration: Duration,
    break_duration: Duration,
    deadline: Option<Instant>,
    remaining: Duration,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub running: bool,
    pub paused: bool,
    pub completed: bool,
    pub phase: &'static str,
    pub cycle: u32,
    pub total: u32,
    pub remaining_seconds: u64,
    pub progress: f64,
}
impl Timer {
    pub fn start(&mut self, settings: &Settings, now: Instant) {
        *self = Self {
            running: true,
            work: true,
            cycle: 1,
            total: settings.cycles,
            work_duration: Duration::from_secs(settings.work_minutes as u64 * 60),
            break_duration: Duration::from_secs(settings.break_minutes as u64 * 60),
            ..Self::default()
        };
        self.deadline = Some(now + self.work_duration);
    }
    pub fn pause(&mut self, now: Instant) {
        if self.running && !self.paused {
            self.remaining = self.deadline.unwrap().saturating_duration_since(now);
            self.paused = true;
        }
    }
    pub fn resume(&mut self, now: Instant) {
        if self.running && self.paused {
            self.deadline = Some(now + self.remaining);
            self.paused = false;
        }
    }
    pub fn advance(&mut self, now: Instant) -> Option<&'static str> {
        let mut message = None;
        while self.running && !self.paused && now >= self.deadline.unwrap() {
            let boundary = self.deadline.unwrap();
            if self.work {
                self.work = false;
                self.deadline = Some(boundary + self.break_duration);
                message = Some("Work session complete. Time for a break.");
            } else if self.cycle == self.total {
                self.running = false;
                self.completed = true;
                self.deadline = None;
                message = Some("All cycles completed. Well done!");
            } else {
                self.cycle += 1;
                self.work = true;
                self.deadline = Some(boundary + self.work_duration);
                message = Some("Break complete. Time to focus.");
            }
        }
        message
    }
    pub fn snapshot(&self, settings: &Settings, now: Instant) -> Snapshot {
        let duration = if self.work {
            self.work_duration
        } else {
            self.break_duration
        };
        let remaining = if self.completed {
            Duration::ZERO
        } else if !self.running {
            Duration::from_secs(settings.work_minutes as u64 * 60)
        } else if self.paused {
            self.remaining
        } else {
            self.deadline.unwrap().saturating_duration_since(now)
        };
        Snapshot {
            running: self.running,
            paused: self.paused,
            completed: self.completed,
            phase: if !self.running || self.work {
                "work"
            } else {
                "break"
            },
            cycle: self.cycle,
            total: if self.running || self.completed {
                self.total
            } else {
                settings.cycles
            },
            remaining_seconds: remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0),
            progress: if self.completed {
                1.0
            } else if !self.running {
                0.0
            } else {
                (1.0 - remaining.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0)
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Settings {
        Settings {
            work_minutes: 1,
            break_minutes: 1,
            cycles: 2,
            ..Settings::default()
        }
    }
    #[test]
    fn catches_up_without_drift() {
        let now = Instant::now();
        let mut t = Timer::default();
        t.start(&config(), now);
        t.advance(now + Duration::from_secs(130));
        let s = t.snapshot(&config(), now + Duration::from_secs(130));
        assert_eq!(s.cycle, 2);
        assert_eq!(s.phase, "work");
        assert_eq!(s.remaining_seconds, 50);
        t.advance(now + Duration::from_secs(240));
        assert!(t.completed);
        assert!(!t.running);
    }
    #[test]
    fn pause_resume_preserves_time() {
        let now = Instant::now();
        let mut t = Timer::default();
        t.start(&config(), now);
        t.pause(now + Duration::from_secs(10));
        t.advance(now + Duration::from_secs(200));
        assert_eq!(t.snapshot(&config(), now).remaining_seconds, 50);
        t.resume(now + Duration::from_secs(200));
        assert_eq!(
            t.snapshot(&config(), now + Duration::from_secs(205))
                .remaining_seconds,
            45
        );
    }
    #[test]
    fn active_settings_are_frozen_and_round_up() {
        let now = Instant::now();
        let mut t = Timer::default();
        t.start(&config(), now);
        let changed = Settings {
            work_minutes: 180,
            ..config()
        };
        let s = t.snapshot(&changed, now + Duration::from_millis(1));
        assert_eq!(s.remaining_seconds, 60);
        assert!(s.progress < 0.001);
    }
    #[test]
    fn reset_clears_completion() {
        let now = Instant::now();
        let mut t = Timer::default();
        t.start(&config(), now);
        t.advance(now + Duration::from_secs(999));
        assert!(t.completed);
        t = Timer::default();
        assert!(!t.snapshot(&config(), now).completed);
    }
}
