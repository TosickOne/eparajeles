// The game side of Presence Footsteps: each tick, the player's feet and the
// block under them (`Entity.getOnPos`: 0.2 below the feet) for the audio side.
use super::Game;
use super::native_hooks::NativeMod;

pub(super) const ID: &str = "presence_footsteps";

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Presence Footsteps 0.1.0".into(),
        description: "Your steps sound like what you walk on.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn presence_footsteps_tick(&mut self) {
        let root = &self.options.resources;
        let player = self.player.as_ref().filter(|_| self.native_on(ID));
        let Some(p) = player else {
            horizon_client::audio::presence_footsteps::player_at(root, None);
            return;
        };
        let feet = [p.position.x, p.position.y, p.position.z];
        let on = [
            feet[0].floor() as i32,
            (feet[1] - 0.2).floor() as i32,
            feet[2].floor() as i32,
        ];
        let block = self.res.facts().get(self.world.block(on[0], on[1], on[2])).name.clone();
        horizon_client::audio::presence_footsteps::player_at(root, Some((feet, &*block, p.sprinting)));
    }
}
