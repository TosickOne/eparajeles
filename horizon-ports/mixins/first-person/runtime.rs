// First-person Model for Horizon: in first person you see your own body
// when you look down. The model is drawn a little behind the camera, so the
// head stays out of view.
use super::Game;
use super::native_hooks::NativeMod;
use std::cell::Cell;

pub(super) const ID: &str = "first_person";
// How far behind the eye the body is drawn, in blocks.
const BACK: f64 = 0.35;

thread_local! {
    // Set while the shifted model is drawn, so the hook lets that call through.
    static DRAWING: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "First-person Model 0.1.0".into(),
        description: "Your body shows when you look down in first person.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    fn first_person_on(&self) -> bool {
        self.native_on(ID) && self.camera_type.first_person() && self.sleeping.is_none()
    }

    // `player_model_shown`, also in first person (the same tail as Horizon's).
    pub(super) fn first_person_shown(&self, shown: bool) -> bool {
        shown
            || (self.first_person_on()
                && !self.spectator()
                && self.potions.effects.get("minecraft:invisibility").is_none()
                && self.player.as_ref().is_some_and(|p| !p.health.dead_or_dying()))
    }

    // `player_model_geometry`'s head: in first person, draw it as seen from a
    // camera moved forward, which puts the body behind the real one.
    pub(super) fn first_person_geometry(
        &self,
        camera: glam::DVec3,
        alpha: f64,
        out: &mut Vec<horizon_client::mesher::Vertex>,
    ) -> core::ops::ControlFlow<()> {
        if DRAWING.with(Cell::get) || !self.first_person_on() {
            return core::ops::ControlFlow::Continue(());
        }
        let (s, c) = f64::from(self.yaw).to_radians().sin_cos();
        let forward = glam::DVec3::new(-s, 0.0, c);
        DRAWING.with(|d| d.set(true));
        self.player_model_geometry(camera + forward * BACK, alpha, out);
        DRAWING.with(|d| d.set(false));
        core::ops::ControlFlow::Break(())
    }
}
