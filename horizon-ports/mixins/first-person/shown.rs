fn shown(value: bool, game: &crate::game::Game<'_>) -> bool {
    game.first_person_shown(value)
}
