fn facts(
    value: Result<(BlockFacts, Vec<GameplayProjection>)>,
    root: &std::path::Path,
) -> Result<(BlockFacts, Vec<GameplayProjection>)> {
    value.map(|(mut facts, projection)| {
        crate::blocks::sounds_mod::retarget(&mut facts, root);
        (facts, projection)
    })
}
