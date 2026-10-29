// A reduced AmbientSounds for Horizon: from the jar's regions (carried over
// by the launcher's mod browser), now and then one of the sounds of the
// regions the player is in plays around them: birds by day in forests,
// crickets at night on plains, drips in caves.
use super::ambient_sounds_engine::{self as engine, Region, Situation, Sound, When};
use super::native_hooks::NativeMod;
use super::Game;
use serde_json::Value;
use std::{cell::RefCell, path::{Path, PathBuf}};

pub(super) const ID: &str = "ambient_sounds";
const REGIONS: &str = "assets/ambientsounds/basic/regions";
// shortcut: one sound at a time, 30 to 60 s apart, at this level; the mod's fades,
// loops, air pockets, features and time curves are not ported.
const VOLUME: f32 = 0.6;
const GAP_TICKS: (u32, u32) = (600, 1200);

#[derive(Default)]
struct State {
    root: PathBuf,
    regions: Vec<Region>,
    wait: u32,
    seed: u64,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "AmbientSounds 0.1.0".into(),
        description: "Birds, crickets, wind and caves around you, from the AmbientSounds jar.".into(),
        keys: Vec::new(),
    }
}

fn strings(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn sound(v: &Value) -> Sound {
    let category = v.get("category").and_then(Value::as_str).unwrap_or("");
    let when = match v.get("time").and_then(Value::as_str) {
        Some("day") => When::Day,
        Some("night") => When::Night,
        _ if category.ends_with("_day") => When::Day,
        _ if category.ends_with("_night") => When::Night,
        _ if v.get("day").and_then(Value::as_f64) == Some(0.0) => When::Night,
        _ => When::Always,
    };
    Sound {
        files: strings(v, "files"),
        volume: v.get("volume").and_then(Value::as_f64).map_or(1.0, |x| x as f32),
        when,
        raining: v.get("overall-raining").and_then(Value::as_bool),
    }
}

fn region(v: &Value) -> Option<Region> {
    let variants = v.get("variants").and_then(Value::as_array);
    let rain = |v: &Value| v.get("raining").and_then(Value::as_bool) == Some(true);
    Some(Region {
        name: v.get("name")?.as_str()?.to_owned(),
        biomes: strings(v, "biomes"),
        bad_biomes: strings(v, "bad-biomes"),
        regions: strings(v, "regions"),
        bad_regions: strings(v, "bad-regions"),
        sky_light_max: v.pointer("/sky-light/max").and_then(Value::as_u64).map(|m| m.min(15) as u8),
        needs_rain: v.get("overall-raining").and_then(Value::as_bool) == Some(true)
            || variants.is_some_and(|vs| vs.iter().any(rain)),
        sounds: v
            .get("sounds")
            .and_then(Value::as_array)
            .map(|s| s.iter().map(sound).collect())
            .unwrap_or_default(),
    })
}

fn load(root: &Path) -> Vec<Region> {
    let Ok(dir) = std::fs::read_dir(root.join(REGIONS)) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = dir.flatten().map(|e| e.path()).collect();
    files.sort();
    files
        .iter()
        .filter_map(|f| {
            let v: Value = serde_json::from_slice(&std::fs::read(f).ok()?).ok()?;
            let r = region(&v);
            if r.is_none() {
                eprintln!("[ambient_sounds] {} has no name; left out", f.display());
            }
            r
        })
        .collect()
}

// A small LCG: the sound picked does not need the game's random sources.
fn next(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 33) as u32
}

impl Game<'_> {
    pub(super) fn ambient_sounds_tick(&mut self) {
        if !self.native_on(ID) || !self.native_window() {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let (x, z) = (p.position.x.floor() as i32, p.position.z.floor() as i32);
        let eye = (p.position.y + f64::from(p.dimensions.eye_height)).floor() as i32;
        let biome = self.world.chunks.get(&(x >> 4, z >> 4)).map_or("minecraft:plains", |c| {
            self.res.biomes().list[c.biome(x, eye, z) as usize].name.as_str()
        });
        let situation = Situation {
            biome: biome.split_once(':').map_or(biome, |(_, path)| path),
            night: (13000..23000).contains(&self.world.day_time.rem_euclid(24000)),
            raining: self.world.weather.levels(1.0).0 > 0.2,
            sky_light: self.world.light(x, eye, z) >> 4,
        };
        let root = &self.options.resources;
        let pick = STATE.with(|s| {
            let mut s = s.borrow_mut();
            if s.as_ref().is_none_or(|s| &s.root != root) {
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |d| d.as_nanos() as u64);
                *s = Some(State { root: root.clone(), regions: load(root), wait: GAP_TICKS.0 / 4, seed });
            }
            let s = s.as_mut().expect("set above");
            if s.wait > 0 {
                s.wait -= 1;
                return None;
            }
            s.wait = GAP_TICKS.0 + next(&mut s.seed) % (GAP_TICKS.1 - GAP_TICKS.0);
            let sounds = engine::playable(&s.regions, &situation);
            if sounds.is_empty() {
                return None;
            }
            let sound = sounds[next(&mut s.seed) as usize % sounds.len()];
            let file = &sound.files[next(&mut s.seed) as usize % sound.files.len()];
            Some((file.clone(), sound.volume))
        });
        if let Some((file, volume)) = pick {
            self.audio.play(&file, None, volume * VOLUME, 1.0);
        }
    }
}
