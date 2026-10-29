// A reduced Visuality: which ambient particle a block near the player gives
// off, and how often. Plain types only, tested without Horizon.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    // Glints on amethyst.
    Sparkle,
    // Souls drifting up from soul sand and soil.
    Soul,
    // Rain rings on open water.
    RainRing,
}

// The effect of `block` (an id) and its chance per sampled tick; `open` is
// air above it, `rain` rain falling on it.
pub fn effect(block: &str, open: bool, rain: bool) -> Option<(Effect, f32)> {
    let path = block.strip_prefix("minecraft:")?;
    match path {
        "amethyst_cluster" | "large_amethyst_bud" | "medium_amethyst_bud" | "small_amethyst_bud" => {
            Some((Effect::Sparkle, 0.5))
        }
        "budding_amethyst" | "amethyst_block" if open => Some((Effect::Sparkle, 0.15)),
        "soul_sand" | "soul_soil" if open => Some((Effect::Soul, 0.08)),
        "water" if open && rain => Some((Effect::RainRing, 0.6)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_by_block_sky_and_rain() {
        assert_eq!(effect("minecraft:amethyst_cluster", false, false).map(|e| e.0), Some(Effect::Sparkle));
        assert_eq!(effect("minecraft:soul_sand", false, false), None);
        assert_eq!(effect("minecraft:soul_soil", true, false).map(|e| e.0), Some(Effect::Soul));
        assert_eq!(effect("minecraft:water", true, false), None);
        assert_eq!(effect("minecraft:water", true, true).map(|e| e.0), Some(Effect::RainRing));
        assert_eq!(effect("minecraft:stone", true, true), None);
    }
}
