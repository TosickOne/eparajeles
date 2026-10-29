fn hand(game: &mut crate::game::Game<'_>, alpha: f64) -> core::ops::ControlFlow<Vec<Vertex>> {
    game.punchy_hand(alpha)
}
