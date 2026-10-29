// CameraOverhaul for Horizon: the camera rolls when strafing or turning and
// pitches when rising, falling or moving forward, on top of the view bob.
use super::Game;
use super::camera_overhaul_engine::Tilt;
use super::native_hooks::NativeMod;
use std::{cell::RefCell, time::Instant};

pub(super) const ID: &str = "camera_overhaul";

thread_local! {
    static TILT: RefCell<(Tilt, Option<Instant>)> = RefCell::new((Tilt::default(), None));
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "CameraOverhaul 0.1.0".into(),
        description: "The camera tilts as you strafe, turn, jump and fall.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn camera_overhaul(&self, bob: glam::Mat4) -> glam::Mat4 {
        let Some(p) = self.player.as_ref().filter(|_| self.native_on(ID)) else {
            return bob;
        };
        let velocity = [p.velocity.x, p.velocity.y, p.velocity.z];
        let (roll, pitch) = TILT.with(|t| {
            let (tilt, last) = &mut *t.borrow_mut();
            let now = Instant::now();
            // A long gap (a pause, a loading screen) starts the tilt over.
            let dt = last.map_or(0.0, |l| now.duration_since(l).as_secs_f32()).min(0.25);
            *last = Some(now);
            tilt.update(dt, self.yaw, velocity)
        });
        bob * glam::Mat4::from_rotation_z(roll.to_radians()) * glam::Mat4::from_rotation_x(pitch.to_radians())
    }
}
