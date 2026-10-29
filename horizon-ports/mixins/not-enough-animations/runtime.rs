// Not Enough Animations for Horizon, at the end of the model's pose set-up:
// players eating or drinking raise the arm to the mouth, and hold a map with
// both arms. Players only, as the mod's poses are.
use super::not_enough_animations_engine::{eat_arm, eating, map_arm};
use super::{Mob, Poses, State, setup};
use std::cell::Cell;

thread_local! {
    // Set while `setup` runs underneath, so the hook lets that call through.
    static POSING: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn pose(
    m: &Mob,
    s: &State,
    p: &mut Poses,
    w_anims: &std::collections::HashMap<String, super::super::keyframes::Animation>,
) -> core::ops::ControlFlow<()> {
    if POSING.with(Cell::get) || m.kind.id != "minecraft:player" {
        return core::ops::ControlFlow::Continue(());
    }
    POSING.with(|c| c.set(true));
    setup(m, s, p, w_anims);
    POSING.with(|c| c.set(false));
    let e = &m.extra;
    // The main hand's arm is the right one unless the player is left-handed.
    let main_right = !m.main_arm_left();
    let (item, right) = if e.using_off_hand {
        (e.off_hand.as_deref(), !main_right)
    } else {
        (e.main_hand.as_deref(), main_right)
    };
    if eating(item, e.using_item) {
        let (x, y) = eat_arm(e.using_item as f32 + s.partial, right);
        let arm = p.get(if right { "right_arm" } else { "left_arm" });
        arm[3] = x;
        arm[4] = y;
    } else if e.using_item == 0 && e.main_hand.as_deref() == Some("minecraft:filled_map") {
        for (name, right) in [("right_arm", true), ("left_arm", false)] {
            let (x, y) = map_arm(right);
            let arm = p.get(name);
            arm[3] = x;
            arm[4] = y;
        }
    }
    core::ops::ControlFlow::Break(())
}
