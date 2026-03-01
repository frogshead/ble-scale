use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ScaleAdvertisement {
    pub weight_kg: f32,
    pub stabilized: bool,
}

/// Parse a MI Scale 2 service data payload (0x181D, 10+ bytes).
///
/// Byte layout:
///   [0]   flags: bit 5 = weight stabilized
///   [1-2] weight little-endian (÷200 = kg)
pub fn parse_advertisement(data: &[u8]) -> Option<ScaleAdvertisement> {
    if data.len() < 3 {
        return None;
    }

    let flags = data[0];
    let stabilized = (flags & (1 << 5)) != 0;

    let raw_weight = (data[2] as u16) << 8 | data[1] as u16;
    let weight_kg = raw_weight as f32 / 200.0;

    Some(ScaleAdvertisement { weight_kg, stabilized })
}

#[derive(Debug)]
enum SessionState {
    Idle,
    Measuring,
    Stabilized,
}

pub enum SessionAction {
    Store(ScaleAdvertisement),
    Ignore,
    SessionEnded,
}

pub struct SessionTracker {
    state: SessionState,
    last_seen: Option<Instant>,
    silence_timeout: Duration,
}

impl SessionTracker {
    pub fn new(silence_timeout: Duration) -> Self {
        SessionTracker {
            state: SessionState::Idle,
            last_seen: None,
            silence_timeout,
        }
    }

    pub fn process(&mut self, adv: ScaleAdvertisement) -> SessionAction {
        self.last_seen = Some(Instant::now());

        match self.state {
            SessionState::Idle => {
                self.state = SessionState::Measuring;
                SessionAction::Ignore
            }
            SessionState::Measuring => {
                if adv.stabilized {
                    self.state = SessionState::Stabilized;
                    SessionAction::Store(adv)
                } else {
                    SessionAction::Ignore
                }
            }
            SessionState::Stabilized => SessionAction::Ignore,
        }
    }

    /// Call periodically (e.g. every 5s) to detect when the scale has gone silent.
    pub fn tick(&mut self) -> SessionAction {
        if let Some(last) = self.last_seen {
            if last.elapsed() >= self.silence_timeout {
                self.state = SessionState::Idle;
                self.last_seen = None;
                return SessionAction::SessionEnded;
            }
        }
        SessionAction::Ignore
    }
}
