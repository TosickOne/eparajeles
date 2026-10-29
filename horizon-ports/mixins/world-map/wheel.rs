fn wheel(game: &mut crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    game.world_map_wheel(steps, claimed);
}
