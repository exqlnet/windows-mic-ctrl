use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use parking_lot::RwLock;

use crate::types::{GateMode, GateState};

#[derive(Debug)]
pub struct GateController {
    is_open: AtomicBool,
    state: RwLock<GateState>,
}

impl GateController {
    pub fn new(initial_mode: GateMode) -> Self {
        let mut state = GateState::default();
        state.mode = initial_mode;
        Self {
            is_open: AtomicBool::new(state.is_open),
            state: RwLock::new(state),
        }
    }

    pub fn set_mode(&self, mode: GateMode) {
        let mut state = self.state.write();
        state.mode = mode;
        state.changed_at = Utc::now();
        state.last_source = "mode".to_string();
    }

    pub fn set_open(&self, open: bool, source: &str) {
        self.is_open.store(open, Ordering::SeqCst);
        let mut state = self.state.write();
        state.is_open = open;
        state.changed_at = Utc::now();
        state.last_source = source.to_string();
    }

    pub fn toggle(&self, source: &str) {
        let next = !self.is_open();
        self.set_open(next, source);
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    pub fn snapshot(&self) -> GateState {
        self.state.read().clone()
    }
}

#[allow(dead_code)]
pub fn apply_envelope(
    current_gain: &mut f32,
    target_open: bool,
    frame_count: usize,
    sample_rate: u32,
) {
    let attack_release_ms = 8.0_f32;
    let step = if sample_rate == 0 {
        1.0
    } else {
        1.0 / ((attack_release_ms / 1000.0) * sample_rate as f32)
    };

    for _ in 0..frame_count {
        if target_open {
            *current_gain = (*current_gain + step).min(1.0);
        } else {
            *current_gain = (*current_gain - step).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_toggle_should_flip_state() {
        let gate = GateController::new(GateMode::Ptt);
        assert!(!gate.is_open());
        gate.toggle("test");
        assert!(gate.is_open());
        gate.toggle("test");
        assert!(!gate.is_open());
    }

    #[test]
    fn envelope_should_reach_bounds() {
        let mut gain = 0.0;
        apply_envelope(&mut gain, true, 960, 48_000);
        assert!(gain > 0.0);
        apply_envelope(&mut gain, false, 9600, 48_000);
        assert!(gain <= 0.01);
    }
}
