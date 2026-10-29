// Presence Footsteps for Horizon: the player's own steps play the mod's
// acoustics for the block underfoot instead of vanilla's step sound, from
// the jar's config and sounds (carried over by the launcher's mod browser).
// The game tells it where the player steps (`player_at`); `intercept` runs
// at the head of `Audio::play`.
use super::Audio;
use super::presence_footsteps_engine::{self as engine, Acoustic, Emit};
use horizon_core::random::RandomSource;
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::ControlFlow,
    path::{Path, PathBuf},
};

const CONFIG: &str = "assets/presencefootsteps/config";
// shortcut: one level for every step; the mod's volume settings are not ported.
const VOLUME: f32 = 0.7;

struct Step {
    feet: [f64; 3],
    block: String,
    running: bool,
}

#[derive(Default)]
struct Config {
    root: PathBuf,
    blocks: HashMap<String, String>,
    primitives: HashMap<String, String>,
    // Each acoustic file, read when first stepped on; None when missing or broken.
    acoustics: HashMap<String, Option<Acoustic>>,
}

thread_local! {
    static STEP: RefCell<Option<Step>> = const { RefCell::new(None) };
    static LOADED: RefCell<Option<Config>> = const { RefCell::new(None) };
}

fn map(root: &Path, file: &str) -> HashMap<String, String> {
    let Ok(bytes) = std::fs::read(root.join(CONFIG).join(file)) else {
        return HashMap::new();
    };
    let Ok(Value::Object(m)) = serde_json::from_slice(&bytes) else {
        eprintln!("[presence_footsteps] {file} does not parse");
        return HashMap::new();
    };
    m.into_iter()
        .filter_map(|(k, v)| Some((k, v.as_str()?.to_owned())))
        .collect()
}

fn range(v: &Value) -> (f32, f32) {
    let get = |k: &str| v.get(k).and_then(Value::as_f64).map_or(100.0, |x| x as f32);
    (get("min"), get("max"))
}

fn acoustic(v: &Value) -> Option<Acoustic> {
    if let Some(all) = v.as_array() {
        return Some(Acoustic::All(all.iter().filter_map(acoustic).collect()));
    }
    let inner = |k: &str| v.get(k).and_then(acoustic).map(Box::new);
    Some(match v.get("type").and_then(Value::as_str).unwrap_or("basic") {
        "basic" => Acoustic::Basic {
            name: v.get("name")?.as_str()?.to_owned(),
            volume: v.get("volume").map_or((100.0, 100.0), range),
            pitch: v.get("pitch").map_or((100.0, 100.0), range),
        },
        "events" => Acoustic::Events(
            v.as_object()?
                .iter()
                .filter(|(k, _)| *k != "type")
                .filter_map(|(k, a)| Some((k.clone(), acoustic(a)?)))
                .collect(),
        ),
        "delayed" => Acoustic::Delayed(inner("acoustic")?),
        "chance" => Acoustic::Chance(v.get("probability")?.as_f64()? as f32, inner("acoustic")?),
        "probability" => Acoustic::Weighted(
            v.get("entries")?
                .as_array()?
                .iter()
                .filter_map(|e| Some((e.get("weight")?.as_f64()? as f32, acoustic(e.get("acoustic")?)?)))
                .collect(),
        ),
        _ => return None,
    })
}

// Where the local player stands this tick and on what; None stops the
// mod (turned off, or no player).
pub fn player_at(root: &Path, step: Option<([f64; 3], &str, bool)>) {
    STEP.with(|s| {
        *s.borrow_mut() = step.map(|(feet, block, running)| Step {
            feet,
            block: block.to_owned(),
            running,
        })
    });
    LOADED.with(|l| {
        let mut l = l.borrow_mut();
        if step.is_some() && l.as_ref().is_none_or(|c| c.root != root) {
            *l = Some(Config {
                root: root.to_owned(),
                blocks: map(root, "blockmap.json"),
                primitives: map(root, "primitivemap.json"),
                acoustics: HashMap::new(),
            });
        }
    });
}

pub(crate) fn intercept(audio: &mut Audio, event: &str, at: Option<[f64; 3]>) -> ControlFlow<()> {
    // Only the local player's step: a `.step` event at the feet `player_at` gave.
    let step = STEP.with(|s| {
        let s = s.borrow();
        let s = s.as_ref()?;
        (at == Some(s.feet) && event.ends_with(".step")).then(|| (s.block.clone(), s.running))
    });
    let Some((block, running)) = step else {
        return ControlFlow::Continue(());
    };
    let mut out = Vec::new();
    let mut rand = || audio.random.next_float();
    let emit = LOADED.with(|l| {
        let mut l = l.borrow_mut();
        let c = l.as_mut()?;
        let value = c.blocks.get(&block).or_else(|| c.primitives.get(event))?;
        let emit = engine::emitter(value);
        if let Emit::Acoustics(names) = &emit {
            for name in names {
                let root = c.root.clone();
                let a = c.acoustics.entry(name.clone()).or_insert_with(|| {
                    let file = root.join(CONFIG).join("acoustics").join(format!("{name}.json"));
                    let v: Value = serde_json::from_slice(&std::fs::read(file).ok()?).ok()?;
                    acoustic(&v)
                });
                if let Some(a) = a {
                    engine::sounds(a, if running { "run" } else { "walk" }, &mut rand, &mut out);
                }
            }
        }
        Some(emit)
    });
    match emit {
        Some(Emit::Silent) => ControlFlow::Break(()),
        Some(Emit::Acoustics(_)) if !out.is_empty() => {
            for s in out {
                let name = format!("presencefootsteps:{}", s.name);
                audio.play(&name, at, s.volume * VOLUME, s.pitch);
            }
            ControlFlow::Break(())
        }
        _ => ControlFlow::Continue(()),
    }
}
