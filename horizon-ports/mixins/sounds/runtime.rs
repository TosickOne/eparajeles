// Sounds for Horizon: when the Sounds jar's assets are in the resource root
// (carried over by the launcher's mod browser), each vanilla block it gives
// sounds of its own breaks, steps, is placed, hit and fallen on with them.
use super::BlockFacts;
use super::sounds_engine::{KINDS, replacement};
use std::{collections::{HashMap, HashSet}, path::Path, sync::Arc};

pub(crate) fn retarget(facts: &mut BlockFacts, root: &Path) {
    let Ok(bytes) = std::fs::read(root.join("assets/sounds/sounds.json")) else {
        return;
    };
    let Ok(serde_json::Value::Object(events)) = serde_json::from_slice(&bytes) else {
        eprintln!("[sounds] assets/sounds/sounds.json does not parse; block sounds stay vanilla's");
        return;
    };
    let events: HashSet<String> = events.keys().cloned().collect();
    // One Arc per event, shared by all the block's states.
    let mut interned: HashMap<String, Arc<str>> = HashMap::new();
    for state in &mut facts.states {
        let sounds = &mut state.sounds;
        let slots = [
            &mut sounds.break_sound,
            &mut sounds.step,
            &mut sounds.place,
            &mut sounds.hit,
            &mut sounds.fall,
        ];
        for (kind, slot) in KINDS.into_iter().zip(slots) {
            if let Some(event) = replacement(&events, &state.name, kind) {
                *slot = interned.entry(event).or_insert_with_key(|e| Arc::from(e.as_str())).clone();
            }
        }
    }
}
