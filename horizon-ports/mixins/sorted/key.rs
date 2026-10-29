fn key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    game.sorted_key(key, pressed);
}
