// Quick Stack: the player slots whose item the container already holds.
// Plain types only, so the tests run without Horizon (`cargo test` in horizon-ports).

pub fn quick_stack<T>(
    player: &[Option<T>],
    container: &[Option<T>],
    same: impl Fn(&T, &T) -> bool,
) -> Vec<usize> {
    player
        .iter()
        .enumerate()
        .filter_map(|(i, stack)| {
            let stack = stack.as_ref()?;
            container
                .iter()
                .flatten()
                .any(|c| same(stack, c))
                .then_some(i)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_only_items_the_container_has() {
        let player = [Some("dirt"), None, Some("stone"), Some("dirt"), Some("apple")];
        let container = [None, Some("dirt"), Some("apple")];
        assert_eq!(quick_stack(&player, &container, |a, b| a == b), [0, 3, 4]);
    }

    #[test]
    fn empty_container_moves_nothing() {
        let player = [Some("dirt")];
        let container: [Option<&str>; 2] = [None, None];
        assert!(quick_stack(&player, &container, |a, b| a == b).is_empty());
    }
}
