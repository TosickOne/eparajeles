// Punchy's visible hand, written from its description: in first person the
// arm shows under the item it holds. Plain types only, tested without Horizon.

// Held items whose first-person pose moves the arm elsewhere (drawn bows and
// crossbows, a raised trident or spear, maps held with both hands): no arm.
pub fn arm_under(item: &str, using: bool) -> bool {
    let busy = using
        && (matches!(item, "minecraft:bow" | "minecraft:crossbow" | "minecraft:trident")
            || item.ends_with("_spear"));
    !busy && item != "minecraft:filled_map" && item != "minecraft:crossbow"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_under_most_items() {
        assert!(arm_under("minecraft:diamond_sword", false));
        assert!(arm_under("minecraft:bow", false));
        assert!(!arm_under("minecraft:bow", true));
        assert!(!arm_under("minecraft:filled_map", false));
        assert!(!arm_under("minecraft:iron_spear", true));
    }
}
