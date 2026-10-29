// A few of Not Enough Animations' third-person poses, written from its
// description: the arm comes up to the mouth while eating or drinking, and
// both arms hold a map out in front. Angles in radians, as HumanoidModel's.
// Plain types only, tested without Horizon.

// Items used with their own vanilla pose, not eaten or drunk.
const OWN_POSE: [&str; 7] = [
    "minecraft:bow",
    "minecraft:crossbow",
    "minecraft:trident",
    "minecraft:shield",
    "minecraft:spyglass",
    "minecraft:goat_horn",
    "minecraft:brush",
];

// Whether a player using `item` for `ticks` is eating or drinking it.
pub fn eating(item: Option<&str>, ticks: i32) -> bool {
    ticks > 0 && item.is_some_and(|i| !OWN_POSE.contains(&i) && !i.ends_with("_spear"))
}

// The using arm's (xRot, yRot) while eating: up to the mouth, turned in,
// bobbing with each bite. `right` is the arm's side.
pub fn eat_arm(ticks: f32, right: bool) -> (f32, f32) {
    let bite = (ticks * 1.6).sin() * 0.12;
    let inward = if right { -0.45 } else { 0.45 };
    (-1.45 + bite, inward)
}

// Each arm's (xRot, yRot) holding a map.
pub fn map_arm(right: bool) -> (f32, f32) {
    (-0.9, if right { -0.35 } else { 0.35 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_but_not_tools_with_their_own_pose() {
        assert!(eating(Some("minecraft:apple"), 3));
        assert!(eating(Some("minecraft:potion"), 1));
        assert!(!eating(Some("minecraft:apple"), 0));
        assert!(!eating(Some("minecraft:bow"), 5));
        assert!(!eating(Some("minecraft:iron_spear"), 5));
        assert!(!eating(None, 5));
    }

    #[test]
    fn arms_raise_forward_and_turn_inward() {
        let (x, y) = eat_arm(0.0, true);
        assert!(x < -1.0 && y < 0.0);
        assert!(eat_arm(0.0, false).1 > 0.0);
        assert_eq!(map_arm(true).0, map_arm(false).0);
    }
}
