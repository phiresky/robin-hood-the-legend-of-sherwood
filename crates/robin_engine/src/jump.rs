//! Runtime jump trajectory state for characters jumping between positions.
//!
//! Tracks the in-flight state of a single jump along a parabolic arc between
//! two 3D positions.

use serde::{Deserialize, Serialize};

/// Runtime state for a character performing a parabolic jump.
///
/// Positions are `(x, y, z)` with `z` as the vertical (height) axis.  During
/// a jump the `x`/`y` coordinates are linearly interpolated while `z` follows
/// a parabolic arc whose apex adds `height` above the linear interpolation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, robin_state_hash_derive::StateHash)]
pub struct Jump {
    /// Start position (x, y, z).
    pub start_pos: (f32, f32, f32),
    /// End position (x, y, z).
    pub end_pos: (f32, f32, f32),
    /// Extra height added at the apex of the parabolic arc.
    pub height: f32,
    /// Progress through the jump: 0.0 = start, 1.0 = landed.
    pub progress: f32,
    /// Whether the jump is currently in progress.
    pub active: bool,
}

impl Jump {
    /// Begin a new jump from `from` to `to` with the given arc `height`.
    ///
    /// Resets progress to 0 and marks the jump active.
    pub fn start(&mut self, from: (f32, f32, f32), to: (f32, f32, f32), height: f32) {
        self.start_pos = from;
        self.end_pos = to;
        self.height = height;
        self.progress = 0.0;
        self.active = true;
    }

    /// Advance the jump by `dt` (a normalised progress increment, typically
    /// `frame_dt / jump_duration`).
    ///
    /// Returns `true` while the jump is still in flight, `false` once landed.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }

        self.progress += dt;

        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.active = false;
            return false;
        }

        true
    }

    /// Current interpolated position along the parabolic arc.
    ///
    /// `x` and `y` are linearly interpolated.  `z` follows a parabola:
    /// `lerp(start_z, end_z, t) + height * 4·t·(1−t)` which peaks at `t = 0.5`.
    pub fn get_current_position(&self) -> (f32, f32, f32) {
        let t = self.progress;
        let x = self.start_pos.0 + (self.end_pos.0 - self.start_pos.0) * t;
        let y = self.start_pos.1 + (self.end_pos.1 - self.start_pos.1) * t;
        let z = self.start_pos.2
            + (self.end_pos.2 - self.start_pos.2) * t
            + self.height * 4.0 * t * (1.0 - t);
        (x, y, z)
    }

    /// Whether the jump is currently in flight.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_sets_fields() {
        let mut j = Jump::default();
        j.start((0.0, 0.0, 0.0), (100.0, 0.0, 50.0), 30.0);

        assert!(j.is_active());
        assert_eq!(j.progress, 0.0);
        assert_eq!(j.start_pos, (0.0, 0.0, 0.0));
        assert_eq!(j.end_pos, (100.0, 0.0, 50.0));
        assert_eq!(j.height, 30.0);
    }

    #[test]
    fn position_at_start_and_end() {
        let mut j = Jump::default();
        j.start((0.0, 0.0, 0.0), (100.0, 200.0, 50.0), 30.0);

        // At t=0 the position should be the start.
        let pos = j.get_current_position();
        assert_eq!(pos, (0.0, 0.0, 0.0));

        // Force to end.
        j.progress = 1.0;
        let pos = j.get_current_position();
        assert!((pos.0 - 100.0).abs() < 1e-5);
        assert!((pos.1 - 200.0).abs() < 1e-5);
        assert!((pos.2 - 50.0).abs() < 1e-5);
    }

    #[test]
    fn apex_at_midpoint() {
        let mut j = Jump::default();
        // Flat jump (same z), height=40 → apex z should be 40.
        j.start((0.0, 0.0, 0.0), (100.0, 0.0, 0.0), 40.0);
        j.progress = 0.5;

        let pos = j.get_current_position();
        assert!((pos.0 - 50.0).abs() < 1e-5);
        // 4 * 0.5 * 0.5 = 1.0 → z = 0 + 40*1 = 40
        assert!((pos.2 - 40.0).abs() < 1e-5);
    }

    #[test]
    fn tick_to_completion() {
        let mut j = Jump::default();
        j.start((0.0, 0.0, 0.0), (100.0, 0.0, 0.0), 20.0);

        // 10 ticks of 0.1 should complete the jump.
        for i in 0..9 {
            assert!(j.tick(0.1), "should still be active at tick {i}");
            assert!(j.is_active());
        }
        // 10th tick lands.
        assert!(!j.tick(0.1));
        assert!(!j.is_active());
        assert_eq!(j.progress, 1.0);

        // Position should be at the end.
        let pos = j.get_current_position();
        assert!((pos.0 - 100.0).abs() < 1e-5);
    }

    #[test]
    fn tick_when_inactive_returns_false() {
        let mut j = Jump::default();
        assert!(!j.tick(0.1));
    }

    #[test]
    fn overshoot_clamps_to_one() {
        let mut j = Jump::default();
        j.start((0.0, 0.0, 0.0), (10.0, 0.0, 0.0), 5.0);
        assert!(!j.tick(1.5));
        assert_eq!(j.progress, 1.0);
    }

    #[test]
    fn serde_roundtrip() {
        let mut j = Jump::default();
        j.start((1.0, 2.0, 3.0), (4.0, 5.0, 6.0), 10.0);
        j.tick(0.3);

        let json = serde_json::to_string(&j).unwrap();
        let j2: Jump = serde_json::from_str(&json).unwrap();

        assert_eq!(j.start_pos, j2.start_pos);
        assert_eq!(j.end_pos, j2.end_pos);
        assert!((j.progress - j2.progress).abs() < 1e-6);
        assert_eq!(j.active, j2.active);
    }
}
