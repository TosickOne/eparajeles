// The game side of Spawn Animations: natural spawning's new mobs rise.
use super::Game;
use super::native_hooks::NativeMod;

pub(super) const ID: &str = "spawn_animations";

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Spawn Animations 0.1.0".into(),
        description: "Mobs that spawn on their own climb out of the ground.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    // `add_spawned`'s head: it gives the next `count` mobs the ids after `next_entity`.
    pub(super) fn spawn_animations_spawned(&mut self, count: usize) {
        if !self.native_on(ID) {
            return;
        }
        let first = self.next_entity + 1;
        let ids = (0..count as u32).map(|i| first + i);
        horizon_client::mobs::render::spawn_animations::spawned(ids);
    }
}
