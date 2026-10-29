// CameraOverhaul's tilts, written from its description: strafing and turning
// roll the camera, rising, falling and moving forward pitch it. Eased each
// frame. Plain types only, tested without Horizon.

// Degrees of roll per block/tick of sideways speed, per degree/second of
// turning; degrees of pitch per block/tick of vertical and forward speed.
const STRAFE_ROLL: f32 = 25.0;
const TURN_ROLL: f32 = 0.012;
const VERTICAL_PITCH: f32 = 6.0;
const FORWARD_PITCH: f32 = 5.0;
const MAX_DEGREES: f32 = 10.0;
// How fast the tilt follows its target (per second).
const EASE: f32 = 8.0;

#[derive(Default)]
pub struct Tilt {
    roll: f32,
    pitch: f32,
    last_yaw: Option<f32>,
}

impl Tilt {
    // One frame `dt` seconds long, the player's yaw (degrees, 0 faces south)
    // and velocity (blocks per tick). Returns (roll, pitch) in degrees.
    pub fn update(&mut self, dt: f32, yaw: f32, velocity: [f64; 3]) -> (f32, f32) {
        let turn = match self.last_yaw {
            Some(last) if dt > 0.0 => {
                let d = (yaw - last + 540.0).rem_euclid(360.0) - 180.0;
                d / dt
            }
            _ => 0.0,
        };
        self.last_yaw = Some(yaw);
        let (s, c) = yaw.to_radians().sin_cos();
        let (vx, vy, vz) = (velocity[0] as f32, velocity[1] as f32, velocity[2] as f32);
        let forward = -s * vx + c * vz;
        let right = -c * vx - s * vz;
        let limit = |v: f32| v.clamp(-MAX_DEGREES, MAX_DEGREES);
        let roll = limit(right * STRAFE_ROLL + turn * TURN_ROLL);
        let pitch = limit(-vy * VERTICAL_PITCH + forward * FORWARD_PITCH);
        let k = 1.0 - (-EASE * dt.max(0.0)).exp();
        self.roll += (roll - self.roll) * k;
        self.pitch += (pitch - self.pitch) * k;
        (self.roll, self.pitch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strafing_rolls_and_standing_still_settles() {
        let mut t = Tilt::default();
        // Facing south, moving west (the player's right).
        let (roll, _) = (0..60).map(|_| t.update(1.0 / 60.0, 0.0, [-0.2, 0.0, 0.0])).last().unwrap();
        assert!(roll > 4.0 && roll <= MAX_DEGREES, "{roll}");
        let (roll, pitch) = (0..240).map(|_| t.update(1.0 / 60.0, 0.0, [0.0; 3])).last().unwrap();
        assert!(roll.abs() < 0.01 && pitch.abs() < 0.01);
    }

    #[test]
    fn falling_pitches_and_turning_across_north_is_short() {
        let mut t = Tilt::default();
        let (_, pitch) = (0..60).map(|_| t.update(1.0 / 60.0, 90.0, [0.0, -0.5, 0.0])).last().unwrap();
        assert!(pitch > 2.0);
        let mut t = Tilt::default();
        t.update(0.1, 179.0, [0.0; 3]);
        let (roll, _) = t.update(0.1, -179.0, [0.0; 3]);
        // 2 degrees in 0.1 s, not 358.
        assert!(roll.abs() < 1.0, "{roll}");
    }
}
