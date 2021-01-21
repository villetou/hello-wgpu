use rand::Rng;
use std::time::{Duration, Instant};

pub enum State {
    Standing {
        duration: Duration,
        started: Instant,
    },
    Walking {
        duration: Duration,
        started: Instant,
        velocity: (f32, f32),
    },
}

pub struct AIController {
    pub state: State,
}

impl AIController {
    pub fn new() -> AIController {
        AIController {
            state: State::Standing {
                duration: Duration::from_secs(2),
                started: Instant::now(),
            },
        }
    }

    pub fn update(&mut self) {
        // Alternates between State::Standing and State::Walking
        match self.state {
            State::Standing { started, duration } => {
                if started.elapsed() > duration {
                    let mut rng = rand::thread_rng();
                    self.state = State::Walking {
                        started: Instant::now(),
                        duration: Duration::from_millis(800 + rng.gen::<u64>() % 1000),
                        velocity: (rng.gen::<f32>() * 2.0 - 1.0, rng.gen::<f32>() * 2.0 - 1.0),
                    }
                }
            }
            State::Walking { started, duration, .. } => {
                if started.elapsed() > duration {
                    let mut rng = rand::thread_rng();
                    self.state = State::Standing {
                        started: Instant::now(),
                        duration: Duration::from_millis(800 + rng.gen::<u64>() % 1000),
                    }
                }
            }
        }
    }
}
