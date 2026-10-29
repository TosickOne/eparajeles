// Spawn Animations' rise, written from its description: a naturally spawned
// mob climbs out of the ground over its first ticks. Plain types only,
// tested without Horizon.

// How long the rise takes, in ticks.
pub const RISE_TICKS: f32 = 20.0;

// How far below its place a mob of `height` is drawn `age` ticks (with the
// frame's fraction) after it spawned: all of it at first, none at the end,
// easing out.
pub fn depth(age: f32, height: f64) -> f64 {
    let t = (age / RISE_TICKS).clamp(0.0, 1.0);
    let left = 1.0 - t;
    f64::from(left * left) * height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rises_from_its_height_to_nothing() {
        assert_eq!(depth(0.0, 2.0), 2.0);
        assert!(depth(10.0, 2.0) < 1.0 && depth(10.0, 2.0) > 0.0);
        assert_eq!(depth(RISE_TICKS, 2.0), 0.0);
        assert_eq!(depth(100.0, 2.0), 0.0);
    }
}
