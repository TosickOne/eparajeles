fn key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    game.world_map_key(key, pressed);
}
