fn command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    game.waystones_command(command, ran);
}
