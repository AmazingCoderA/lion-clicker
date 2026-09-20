use crate::{
    backend::{HeldMouse, MouseOutput, NativeMouse},
    config::{Config, Distribution},
};
use anyhow::{Context, Result};
use rand::Rng;
use rand_distr::{Distribution as _, StandardNormal};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Countdown,
    Clicking,
    Break,
}

#[derive(Clone, Default)]
pub struct Status {
    pub phase: Phase,
    pub clicks: u64,
    pub elapsed: Duration,
    pub error: Option<String>,
    started: Option<Instant>,
}

impl Status {
    pub fn running(&self) -> bool {
        self.phase != Phase::Idle
    }
}

enum Command {
    Start(Config),
    Stop,
    Shutdown,
}

pub struct Engine {
    tx: mpsc::Sender<Command>,
    status: Arc<Mutex<Status>>,
    worker: Option<JoinHandle<()>>,
}

impl Engine {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let status = Arc::new(Mutex::new(Status::default()));
        let shared = status.clone();
        let worker = thread::Builder::new()
            .name("lion-clicker".into())
            .spawn(move || {
                let mut control = Control {
                    rx,
                    pending: None,
                    deadline: None,
                };
                loop {
                    let command = control.pending.take().or_else(|| control.rx.recv().ok());
                    match command {
                        Some(Command::Start(config)) => {
                            *shared.lock().unwrap() = Status {
                                phase: Phase::Countdown,
                                ..Status::default()
                            };
                            let result = (|| {
                                config.validate()?;
                                // Give the user time to move away from the Start button.
                                if control.wait(Duration::from_millis(config.start_delay_ms)) {
                                    return Ok(());
                                }
                                let mouse = NativeMouse::new()?;
                                run_session(&config, mouse, &mut control, &shared)
                            })();
                            control.deadline = None;
                            let mut state = shared.lock().unwrap();
                            state.phase = Phase::Idle;
                            state.started = None;
                            if let Err(error) = result {
                                state.error = Some(format!("{error:#}"));
                            }
                        }
                        Some(Command::Stop) => shared.lock().unwrap().phase = Phase::Idle,
                        Some(Command::Shutdown) | None => break,
                    }
                }
            })
            .context("Cannot start clicker worker")?;
        Ok(Self {
            tx,
            status,
            worker: Some(worker),
        })
    }

    pub fn start(&self, config: Config) -> Result<()> {
        config.validate()?;
        let mut status = self.status.lock().unwrap();
        *status = Status {
            phase: Phase::Countdown,
            ..Status::default()
        };
        if let Err(error) = self.tx.send(Command::Start(config)) {
            status.phase = Phase::Idle;
            return Err(error).context("Clicker worker has stopped");
        }
        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Command::Stop);
    }

    pub fn status(&self) -> Status {
        let mut status = self.status.lock().unwrap().clone();
        if let Some(started) = status.started {
            status.elapsed = started.elapsed();
        }
        status
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct Control {
    rx: mpsc::Receiver<Command>,
    pending: Option<Command>,
    deadline: Option<Instant>,
}

impl Control {
    /// All waits (including mouse-down and long breaks) wake on a command.
    fn wait(&mut self, duration: Duration) -> bool {
        if self.pending.is_some() {
            return true;
        }
        let duration = if let Some(deadline) = self.deadline {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return true;
            };
            duration.min(remaining)
        } else {
            duration
        };
        match self.rx.recv_timeout(duration) {
            Ok(command) => {
                self.pending = Some(command);
                true
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.pending = Some(Command::Shutdown);
                true
            }
            Err(mpsc::RecvTimeoutError::Timeout) => self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline),
        }
    }
}

pub fn sample_ms(rng: &mut impl Rng, base: u64, jitter: u64, distribution: Distribution) -> u64 {
    if jitter == 0 {
        return base.max(1);
    }
    let offset = match distribution {
        Distribution::Uniform => rng.gen_range(-(jitter as f64)..=jitter as f64),
        Distribution::Normal => {
            let value: f64 = StandardNormal.sample(rng);
            (value * jitter as f64 / 3.0).clamp(-(jitter as f64), jitter as f64)
        }
    };
    (base as f64 + offset).round().max(1.0) as u64
}

fn run_session<M: MouseOutput>(
    config: &Config,
    mouse: M,
    control: &mut Control,
    shared: &Mutex<Status>,
) -> Result<()> {
    let mut mouse = HeldMouse::new(mouse);
    let mut rng = rand::thread_rng();
    let started = Instant::now();
    shared.lock().unwrap().started = Some(started);
    control.deadline =
        (config.max_duration_s > 0).then(|| started + Duration::from_secs(config.max_duration_s));
    let jitter = |value| if config.randomize { value } else { 0 };
    let mut clicks = 0;
    let result = (|| {
        'session: loop {
            shared.lock().unwrap().phase = Phase::Clicking;
            for index in 0..config.sequence_len() {
                if control.wait(Duration::ZERO) {
                    break 'session;
                }
                let restore_position = if config.fixed_position {
                    let radius = if config.randomize {
                        config.radius_px as i32
                    } else {
                        0
                    };
                    // Offsets are always relative to the configured point, never a random walk.
                    let dx = rng.gen_range(-radius..=radius);
                    let dy = rng.gen_range(-radius..=radius);
                    mouse.output.move_to(config.x + dx, config.y + dy)?;
                    None
                } else if config.randomize && config.cursor_tremor && config.tremor_px > 0 {
                    let (x, y) = mouse.output.position()?;
                    let radius = config.tremor_px as i32;
                    let dx = rng.gen_range(-radius..=radius);
                    let dy = rng.gen_range(-radius..=radius);
                    mouse.output.move_to(x + dx, y + dy)?;
                    Some((x, y))
                } else {
                    None
                };
                mouse.press(config.button)?;
                let hold = sample_ms(
                    &mut rng,
                    config.hold_ms,
                    jitter(config.hold_jitter_ms),
                    config.distribution,
                );
                let interrupted = control.wait(Duration::from_millis(hold));
                mouse.release()?;
                if let Some((x, y)) = restore_position {
                    mouse.output.move_to(x, y)?;
                }
                clicks += 1;
                {
                    let mut state = shared.lock().unwrap();
                    state.clicks = clicks;
                    state.elapsed = started.elapsed();
                }
                if interrupted || (config.max_clicks > 0 && clicks >= config.max_clicks) {
                    break 'session;
                }
                if index + 1 < config.sequence_len() {
                    let interval = sample_ms(
                        &mut rng,
                        config.burst_interval_ms,
                        jitter(config.burst_jitter_ms),
                        config.distribution,
                    );
                    if control.wait(Duration::from_millis(interval)) {
                        break 'session;
                    }
                }
            }
            if config.randomize && config.breaks && rng.gen_range(0..100) < config.break_probability
            {
                shared.lock().unwrap().phase = Phase::Break;
                let pause = rng.gen_range(config.break_min_ms..=config.break_max_ms);
                if control.wait(Duration::from_millis(pause)) {
                    break;
                }
            }
            let delay = sample_ms(
                &mut rng,
                config.delay_ms,
                jitter(config.delay_jitter_ms),
                config.distribution,
            );
            if control.wait(Duration::from_millis(delay)) {
                break;
            }
        }
        Ok(())
    })();
    shared.lock().unwrap().elapsed = started.elapsed();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Button, Pattern};
    use rand::{rngs::StdRng, SeedableRng};

    struct FakeMouse {
        events: Arc<Mutex<Vec<&'static str>>>,
        cancel_on_press: Option<mpsc::Sender<Command>>,
        fail_press: bool,
    }

    impl MouseOutput for FakeMouse {
        fn press(&mut self, _: Button) -> Result<()> {
            self.events.lock().unwrap().push("down");
            if let Some(tx) = &self.cancel_on_press {
                tx.send(Command::Stop).unwrap();
            }
            if self.fail_press {
                anyhow::bail!("injection failed");
            }
            Ok(())
        }
        fn release(&mut self, _: Button) -> Result<()> {
            self.events.lock().unwrap().push("up");
            Ok(())
        }
        fn position(&self) -> Result<(i32, i32)> {
            Ok((100, 200))
        }
        fn move_to(&mut self, _: i32, _: i32) -> Result<()> {
            self.events.lock().unwrap().push("move");
            Ok(())
        }
    }

    #[test]
    fn stop_interrupts_long_hold_and_releases_button() {
        let (tx, rx) = mpsc::channel();
        let events = Arc::new(Mutex::new(vec![]));
        let mouse = FakeMouse {
            events: events.clone(),
            cancel_on_press: Some(tx),
            fail_press: false,
        };
        let config = Config {
            hold_ms: 60_000,
            randomize: false,
            ..Config::default()
        };
        let mut control = Control {
            rx,
            pending: None,
            deadline: None,
        };
        let started = Instant::now();
        run_session(&config, mouse, &mut control, &Mutex::new(Status::default())).unwrap();
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(*events.lock().unwrap(), ["down", "up"]);
        assert!(matches!(control.pending, Some(Command::Stop)));
    }

    #[test]
    fn partial_press_failure_still_releases() {
        let (_tx, rx) = mpsc::channel();
        let events = Arc::new(Mutex::new(vec![]));
        let mouse = FakeMouse {
            events: events.clone(),
            cancel_on_press: None,
            fail_press: true,
        };
        let mut control = Control {
            rx,
            pending: None,
            deadline: None,
        };
        assert!(run_session(
            &Config::default(),
            mouse,
            &mut control,
            &Mutex::new(Status::default())
        )
        .is_err());
        assert_eq!(*events.lock().unwrap(), ["down", "up"]);
    }

    #[test]
    fn click_limit_can_end_in_middle_of_burst() {
        let (_tx, rx) = mpsc::channel();
        let events = Arc::new(Mutex::new(vec![]));
        let mouse = FakeMouse {
            events: events.clone(),
            cancel_on_press: None,
            fail_press: false,
        };
        let config = Config {
            pattern: Pattern::Burst,
            burst_count: 20,
            max_clicks: 2,
            hold_ms: 1,
            burst_interval_ms: 1,
            randomize: false,
            ..Config::default()
        };
        let shared = Mutex::new(Status::default());
        let mut control = Control {
            rx,
            pending: None,
            deadline: None,
        };
        run_session(&config, mouse, &mut control, &shared).unwrap();
        assert_eq!(shared.lock().unwrap().clicks, 2);
        assert_eq!(*events.lock().unwrap(), ["down", "up", "down", "up"]);
    }

    #[test]
    fn cursor_tremor_restores_original_position() {
        let (_tx, rx) = mpsc::channel();
        let events = Arc::new(Mutex::new(vec![]));
        let mouse = FakeMouse {
            events: events.clone(),
            cancel_on_press: None,
            fail_press: false,
        };
        let config = Config {
            cursor_tremor: true,
            tremor_px: 2,
            max_clicks: 1,
            hold_ms: 1,
            randomize: true,
            ..Config::default()
        };
        let mut control = Control {
            rx,
            pending: None,
            deadline: None,
        };
        run_session(&config, mouse, &mut control, &Mutex::new(Status::default())).unwrap();
        assert_eq!(*events.lock().unwrap(), ["move", "down", "up", "move"]);
    }

    #[test]
    fn deadline_interrupts_wait_and_equal_break_bounds_work() {
        let (_tx, rx) = mpsc::channel();
        let mut control = Control {
            rx,
            pending: None,
            deadline: Some(Instant::now() + Duration::from_millis(10)),
        };
        assert!(control.wait(Duration::from_secs(60)));
        let mut rng = StdRng::seed_from_u64(7);
        assert_eq!(rng.gen_range(200..=200), 200);
        for distribution in [Distribution::Uniform, Distribution::Normal] {
            for _ in 0..10_000 {
                assert!((25..=55).contains(&sample_ms(&mut rng, 40, 15, distribution)));
                assert!((1..=20).contains(&sample_ms(&mut rng, 5, 15, distribution)));
            }
        }
    }
}
