// Sounds' per-block sound events: `sounds:block.<block>.<kind>` stands in for
// the vanilla event of that block and kind when the mod's sounds.json has it.
// Plain types only, tested without Horizon.
use std::collections::HashSet;

pub const KINDS: [&str; 5] = ["break", "step", "place", "hit", "fall"];

pub fn replacement(events: &HashSet<String>, block: &str, kind: &str) -> Option<String> {
    let path = block.strip_prefix("minecraft:")?;
    let event = format!("block.{path}.{kind}");
    events.contains(&event).then(|| format!("sounds:{event}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_vanilla_blocks_the_mod_covers() {
        let events: HashSet<String> = ["block.oak_log.step".to_owned()].into();
        assert_eq!(
            replacement(&events, "minecraft:oak_log", "step").as_deref(),
            Some("sounds:block.oak_log.step")
        );
        assert_eq!(replacement(&events, "minecraft:oak_log", "break"), None);
        assert_eq!(replacement(&events, "minecraft:stone", "step"), None);
        assert_eq!(replacement(&events, "other:oak_log", "step"), None);
    }
}
