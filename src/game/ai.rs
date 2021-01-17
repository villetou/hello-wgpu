use std::time::{Instant, Duration};

pub enum State {
    Standing {
        duration: Duration,
        started: Instant,
    },
    Walking {
        duration: Duration,
        started: Instant,
        velocity: f32,
    }
}

pub struct AIController {
    state: State,
}

impl AIController {
    pub fn update(&mut self) {
        // Alternates between State::Standing and State::Walking
        match self.state {
            State::Standing {started, duration} => {
                if started.elapsed() > duration {
                    self.state = State::Walking{
                        started: Instant::now(),
                        duration: Duration::from_secs(1),
                        velocity: 1.0,
                    }
                }
            },
            State::Walking {started, duration, ..} => {
                if started.elapsed() > duration {
                    self.state = State::Standing{
                        started: Instant::now(),
                        duration: Duration::from_secs(1),
                    }
                }
            },
        }
    }
}
