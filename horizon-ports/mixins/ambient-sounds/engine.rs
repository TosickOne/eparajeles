// A reduced AmbientSounds: which of the mod's regions (regions/*.json) are
// active where the player stands, and which of their sounds may play.
// Plain types only, tested without Horizon.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum When {
    Always,
    Day,
    Night,
}

#[derive(Debug, Default)]
pub struct Region {
    pub name: String,
    // Biome name patterns (`mushroom*fields`, `rocky plateau`); `#tags` are left out.
    pub biomes: Vec<String>,
    pub bad_biomes: Vec<String>,
    // Regions that must be active too, and ones that must not.
    pub regions: Vec<String>,
    pub bad_regions: Vec<String>,
    // A cave region: the sky light at the player at most this.
    pub sky_light_max: Option<u8>,
    pub needs_rain: bool,
    pub sounds: Vec<Sound>,
}

#[derive(Debug)]
pub struct Sound {
    pub files: Vec<String>,
    pub volume: f32,
    pub when: When,
    // `overall-raining`: Some(false) plays only in dry weather.
    pub raining: Option<bool>,
}

pub struct Situation<'a> {
    // The biome's id path, `flower_forest`.
    pub biome: &'a str,
    pub night: bool,
    pub raining: bool,
    pub sky_light: u8,
}

pub fn biome_matches(pattern: &str, biome: &str) -> bool {
    if pattern.starts_with('#') {
        return false;
    }
    let biome = biome.replace('_', " ");
    let pattern = pattern.replace('_', " ").to_lowercase();
    let mut rest = biome.as_str();
    for part in pattern.split('*') {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    true
}

// The names of the active regions. `surface` (no biome list, no cave rule)
// counts as open sky; other regions without either never apply.
pub fn active<'r>(regions: &'r [Region], s: &Situation) -> Vec<&'r str> {
    let base = |r: &Region| {
        let biome = !r.biomes.is_empty()
            && r.biomes.iter().any(|p| biome_matches(p, s.biome))
            && !r.bad_biomes.iter().any(|p| biome_matches(p, s.biome));
        let cave = r.sky_light_max.is_some_and(|max| s.sky_light <= max);
        let surface = r.name == "surface" && s.sky_light >= 8;
        (biome || cave || surface || (r.biomes.is_empty() && r.sky_light_max.is_none() && !r.regions.is_empty()))
            && (!r.needs_rain || s.raining)
    };
    let first: Vec<&str> = regions.iter().filter(|r| base(r)).map(|r| r.name.as_str()).collect();
    regions
        .iter()
        .filter(|r| {
            first.contains(&r.name.as_str())
                && r.regions.iter().all(|n| first.contains(&n.as_str()))
                && !r.bad_regions.iter().any(|n| first.contains(&n.as_str()))
        })
        .map(|r| r.name.as_str())
        .collect()
}

// The sounds that may play now in the active regions.
pub fn playable<'r>(regions: &'r [Region], s: &Situation) -> Vec<&'r Sound> {
    let names = active(regions, s);
    regions
        .iter()
        .filter(|r| names.contains(&r.name.as_str()))
        .flat_map(|r| &r.sounds)
        .filter(|x| match x.when {
            When::Always => true,
            When::Day => !s.night,
            When::Night => s.night,
        })
        .filter(|x| x.raining.is_none_or(|r| r == s.raining))
        .filter(|x| !x.files.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sound(file: &str, when: When) -> Sound {
        Sound { files: vec![file.into()], volume: 1.0, when, raining: Some(false) }
    }

    #[test]
    fn biome_patterns() {
        assert!(biome_matches("mushroom*fields", "mushroom_fields"));
        assert!(biome_matches("forest", "flower_forest"));
        assert!(biome_matches("rocky plateau", "rocky_plateau"));
        assert!(!biome_matches("#is_forest", "forest"));
        assert!(!biome_matches("plains", "forest"));
    }

    #[test]
    fn regions_by_biome_time_rain_and_bad_regions() {
        let regions = vec![
            Region { name: "surface".into(), ..Default::default() },
            Region {
                name: "plains".into(),
                biomes: vec!["plains".into()],
                bad_regions: vec!["forest".into()],
                sounds: vec![sound("cricket", When::Day), sound("owl", When::Night)],
                ..Default::default()
            },
            Region { name: "forest".into(), biomes: vec!["forest".into()], ..Default::default() },
            Region { name: "rain".into(), regions: vec!["surface".into()], needs_rain: true, ..Default::default() },
        ];
        let day = Situation { biome: "plains", night: false, raining: false, sky_light: 15 };
        assert_eq!(active(&regions, &day), ["surface", "plains"]);
        let names: Vec<_> = playable(&regions, &day).iter().map(|s| s.files[0].as_str()).collect();
        assert_eq!(names, ["cricket"]);
        let wet = Situation { raining: true, ..day };
        assert_eq!(active(&regions, &wet), ["surface", "plains", "rain"]);
        assert!(playable(&regions, &wet).is_empty());
        let woods = Situation { biome: "sunflower_plains_forest", ..day };
        assert_eq!(active(&regions, &woods), ["surface", "forest"]);
    }
}
