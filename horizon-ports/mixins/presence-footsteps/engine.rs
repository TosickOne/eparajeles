// Presence Footsteps' acoustics: what a step on a block plays, read from the
// mod's own config files (blockmap, primitivemap, acoustics/*.json). Plain
// types only, tested without Horizon.

pub enum Acoustic {
    // A sound event of the mod; volume and pitch ranges in percent.
    Basic { name: String, volume: (f32, f32), pitch: (f32, f32) },
    // One acoustic per locomotion event (walk, run, wander, land, jump...).
    Events(Vec<(String, Acoustic)>),
    All(Vec<Acoustic>),
    Delayed(Box<Acoustic>),
    Weighted(Vec<(f32, Acoustic)>),
    // Plays the inner acoustic with this chance, in percent.
    Chance(f32, Box<Acoustic>),
}

#[derive(Debug, PartialEq)]
pub struct Sound {
    pub name: String,
    pub volume: f32,
    pub pitch: f32,
}

// What a blockmap or primitivemap value asks for.
#[derive(Debug, PartialEq)]
pub enum Emit {
    Silent,
    Vanilla,
    Acoustics(Vec<String>),
}

pub fn emitter(value: &str) -> Emit {
    match value {
        "NOT_EMITTER" | "NON_EMITTER" | "NOT_EMITTED" => Emit::Silent,
        // shortcut: MESSY_GROUND (the block below decides) plays vanilla's step, as VANILLA does.
        "VANILLA" | "MESSY_GROUND" | "" => Emit::Vanilla,
        list => Emit::Acoustics(list.split(',').map(str::to_owned).collect()),
    }
}

// The sounds `acoustic` plays for locomotion `event`; `rand` gives 0..1.
pub fn sounds(acoustic: &Acoustic, event: &str, rand: &mut impl FnMut() -> f32, out: &mut Vec<Sound>) {
    let between = |(lo, hi): (f32, f32), r: f32| (lo + (hi - lo) * r) / 100.0;
    match acoustic {
        Acoustic::Basic { name, volume, pitch } => {
            if !name.is_empty() {
                let volume = between(*volume, rand());
                let pitch = between(*pitch, rand());
                out.push(Sound { name: name.clone(), volume, pitch });
            }
        }
        Acoustic::Events(events) => {
            let pick = |want: &str| events.iter().find(|(e, _)| e == want).map(|(_, a)| a);
            // A run falls back to the walk, as a missing event plays nothing else.
            if let Some(a) = pick(event).or_else(|| pick("walk")) {
                sounds(a, event, rand, out);
            }
        }
        Acoustic::All(all) => {
            for a in all {
                sounds(a, event, rand, out);
            }
        }
        // shortcut: delayed parts (a landing's second thud) are left out until steps can schedule sounds.
        Acoustic::Delayed(_) => {}
        Acoustic::Weighted(entries) => {
            let total: f32 = entries.iter().map(|(w, _)| w).sum();
            let mut at = rand() * total;
            for (w, a) in entries {
                if at < *w {
                    sounds(a, event, rand, out);
                    return;
                }
                at -= w;
            }
        }
        Acoustic::Chance(percent, a) => {
            if rand() * 100.0 < *percent {
                sounds(a, event, rand, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic(name: &str) -> Acoustic {
        Acoustic::Basic { name: name.into(), volume: (50.0, 100.0), pitch: (100.0, 100.0) }
    }

    #[test]
    fn emitter_values() {
        assert_eq!(emitter("NOT_EMITTER"), Emit::Silent);
        assert_eq!(emitter("VANILLA"), Emit::Vanilla);
        assert_eq!(emitter("stone,ore"), Emit::Acoustics(vec!["stone".into(), "ore".into()]));
    }

    #[test]
    fn events_pick_the_locomotion_and_fall_back_to_walk() {
        let wood = Acoustic::Events(vec![
            ("walk".into(), basic("wood_walk")),
            ("land".into(), Acoustic::All(vec![basic("wood_land"), Acoustic::Delayed(Box::new(basic("late")))])),
        ]);
        let mut out = Vec::new();
        sounds(&wood, "run", &mut || 0.5, &mut out);
        assert_eq!(out, [Sound { name: "wood_walk".into(), volume: 0.75, pitch: 1.0 }]);
        out.clear();
        sounds(&wood, "land", &mut || 0.0, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "wood_land");
    }

    #[test]
    fn weights_chances_and_silent_names() {
        let a = Acoustic::Weighted(vec![(1.0, basic("a")), (3.0, basic(""))]);
        let mut out = Vec::new();
        sounds(&a, "walk", &mut || 0.1, &mut out);
        assert_eq!(out[0].name, "a");
        out.clear();
        sounds(&a, "walk", &mut || 0.9, &mut out);
        assert!(out.is_empty());
        sounds(&Acoustic::Chance(40.0, Box::new(basic("c"))), "walk", &mut || 0.5, &mut out);
        assert!(out.is_empty());
    }
}
