fn use_block(
    game: &mut crate::game::Game<'_>,
    hit: Option<&crate::game::Hit>,
) -> core::ops::ControlFlow<Result<bool>> {
    game.waystones_use(hit)
}
