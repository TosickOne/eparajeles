// Punchy for Horizon: the first-person hand draws the main-hand arm under the
// item it holds, posed and swinging as the empty arm does
// (`renderPlayerArm`), which puts the hand under vanilla's item.
use super::native_hooks::NativeMod;
use super::punchy_engine::arm_under;
use super::{Game, Hand, Vertex};
use std::cell::Cell;

pub(super) const ID: &str = "punchy";

thread_local! {
    // Set while Horizon's own hand geometry runs underneath.
    static DRAWING: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Punchy 0.1.0".into(),
        description: "Your arm shows under the item you hold in first person.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn punchy_hand(&mut self, alpha: f64) -> core::ops::ControlFlow<Vec<Vertex>> {
        if DRAWING.with(Cell::get) || !self.native_on(ID) {
            return core::ops::ControlFlow::Continue(());
        }
        DRAWING.with(|d| d.set(true));
        let mut out = self.hand_geometry(alpha);
        DRAWING.with(|d| d.set(false));
        // Horizon drew nothing (third person, F1, dead...) or the main hand is hidden.
        if out.is_empty() || !self.hands_to_render()[0] {
            return core::ops::ControlFlow::Break(out);
        }
        let Some(stack) = self.visible_hand_items()[0].clone() else {
            // An empty main hand already shows its arm.
            return core::ops::ControlFlow::Break(out);
        };
        let using = self.used_hand() == Some(Hand::Main);
        if !arm_under(&stack.item, using) {
            return core::ops::ControlFlow::Break(out);
        }
        let a = alpha as f32;
        let attack = if self.swing.attack_hand() == Hand::Main { self.swing.animation(a) } else { 0.0 };
        let inverse_height = 1.0 - self.heights.interpolated(a)[0];
        let view = horizon_client::camera::view_matrix(self.yaw, self.pitch);
        let [sway_x, sway_y] = self.sway.rotation(a, self.pitch, self.yaw);
        let to_world = view.inverse()
            * glam::Mat4::from_rotation_x(sway_x.to_radians())
            * glam::Mat4::from_rotation_y(sway_y.to_radians());
        let eye = self.eye_position().unwrap_or_default();
        let light = self.world.light(eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32);
        let packed = (u32::from(light >> 4) * 16) | ((u32::from(light & 15) * 16) << 8);
        let slim = self.player_model.as_ref().is_some_and(|m| m.slim);
        let Some(arm) = self.res.player_arm(slim).clone() else {
            return core::ops::ControlFlow::Break(out);
        };
        let sprites = &self.gui.atlas.sprites;
        let Some(r) = self
            .player_model
            .as_ref()
            .and_then(|m| m.skin.as_deref())
            .and_then(|skin| sprites.get(skin))
            .or_else(|| sprites.get("entity/player/wide/steve"))
            .copied()
        else {
            return core::ops::ControlFlow::Break(out);
        };
        // renderPlayerArm, as Horizon draws the empty main hand.
        use std::f32::consts::{PI, TAU};
        let deg = |d: f32| d.to_radians();
        let sqrt_attack = attack.sqrt();
        let x_swing = -0.3 * (sqrt_attack * PI).sin();
        let y_swing = 0.4 * (sqrt_attack * TAU).sin();
        let z_swing = -0.4 * (attack * PI).sin();
        let z_rot = (attack * attack * PI).sin();
        let y_rot = (sqrt_attack * PI).sin();
        let m = to_world
            * glam::Mat4::from_translation(glam::Vec3::new(
                x_swing + 0.64,
                y_swing - 0.6 + inverse_height * -0.6,
                z_swing - 0.72,
            ))
            * glam::Mat4::from_rotation_y(deg(45.0))
            * glam::Mat4::from_rotation_y(deg(y_rot * 70.0))
            * glam::Mat4::from_rotation_z(deg(z_rot * -20.0))
            * glam::Mat4::from_translation(glam::Vec3::new(-1.0, 3.6, 3.5))
            * glam::Mat4::from_rotation_z(deg(120.0))
            * glam::Mat4::from_rotation_x(deg(200.0))
            * glam::Mat4::from_rotation_y(deg(-135.0))
            * glam::Mat4::from_translation(glam::Vec3::new(5.6, 0.0, 0.0));
        let pose = |p: &horizon_client::mobs::Part| {
            let mut pose = p.pose;
            pose[3] = 0.0;
            pose[4] = 0.0;
            pose[5] = 0.0;
            pose
        };
        horizon_client::mobs::render_model(&arm, m, &pose, [r.u0, r.v0, r.u1, r.v1], packed, 0, &mut out);
        core::ops::ControlFlow::Break(out)
    }
}
