fn bob(value: glam::Mat4, game: &crate::game::Game<'_>) -> glam::Mat4 {
    game.camera_overhaul(value)
}
