// Spawn Animations for Horizon, in the library's mob renderer: mobs the
// game marked as just spawned are drawn lower, rising out of the ground.
use super::super::{Mob, MobData, Part};
use super::{Atlas, render_with};
use crate::mesher::Vertex;
use std::{cell::{Cell, RefCell}, collections::HashSet};

thread_local! {
    static RISING: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
    // Set while the lowered mob is drawn, so the hook lets that call through.
    static DRAWING: Cell<bool> = const { Cell::new(false) };
}

// The game's natural spawning names the mobs it just added.
pub fn spawned(ids: impl IntoIterator<Item = u32>) {
    RISING.with(|r| r.borrow_mut().extend(ids));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    m: &Mob,
    alpha: f32,
    camera: glam::DVec3,
    atlas: Atlas,
    light: u32,
    data: &MobData,
    extra: &[(&Part, [f32; 4], u32)],
    out: &mut Vec<Vertex>,
) -> core::ops::ControlFlow<()> {
    if DRAWING.with(Cell::get) || !RISING.with(|r| r.borrow().contains(&m.id)) {
        return core::ops::ControlFlow::Continue(());
    }
    let age = m.age as f32 + alpha;
    if age >= super::spawn_animations_engine::RISE_TICKS {
        RISING.with(|r| r.borrow_mut().remove(&m.id));
        return core::ops::ControlFlow::Continue(());
    }
    let depth = super::spawn_animations_engine::depth(age, m.kind.height);
    // A camera raised by `depth` draws the mob that much lower.
    DRAWING.with(|d| d.set(true));
    render_with(m, alpha, camera + glam::DVec3::Y * depth, atlas, light, data, extra, out);
    DRAWING.with(|d| d.set(false));
    core::ops::ControlFlow::Break(())
}
