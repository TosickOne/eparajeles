// Waystones for Horizon, in single player: using a waystone block (from the
// Waystones jar carried over by the launcher) activates it, and
// `/waystones <n>` near an activated one travels to the n-th. `/waystones`
// lists them.
use super::native_hooks::NativeMod;
use super::waystones_engine::{self as engine, Waystone};
use super::{Game, Hit, Result};
use std::{ops::ControlFlow, path::PathBuf};
use winit::keyboard::KeyCode;

pub(super) const ID: &str = "waystones";
// How close to an activated waystone a player must stand to travel.
const REACH: f64 = 4.0;

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Waystones 0.1.0".into(),
        description: "Use a waystone to activate it; /waystones lists them, /waystones <n> travels."
            .into(),
        keys: Vec::new(),
    }
}

// `waystones:waystone` and its stone variants; not the sharestones, portstones or warp plates.
fn is_waystone(block: &str) -> bool {
    block
        .strip_prefix("waystones:")
        .is_some_and(|b| b == "waystone" || b.ends_with("_waystone"))
}

impl Game<'_> {
    fn waystones_file(&self) -> Option<PathBuf> {
        let world = self.world_root.file_name()?.to_string_lossy().into_owned();
        Some(self.native_data_dir(ID).join(world).join("waystones.txt"))
    }

    fn waystones_load(&self) -> Vec<Waystone> {
        self.waystones_file()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .map_or_else(Vec::new, |t| engine::parse(&t))
    }

    fn waystones_save(&mut self, stones: &[Waystone]) {
        let Some(file) = self.waystones_file() else {
            return;
        };
        let tmp = file.with_extension("tmp");
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&tmp, engine::write(stones)))
            .and_then(|()| std::fs::rename(&tmp, &file));
        if let Err(e) = written {
            self.native_chat(&format!("Waystones: could not save {}: {e}", file.display()));
        }
    }

    fn waystones_list(&mut self, stones: &[Waystone]) {
        if stones.is_empty() {
            self.native_chat("No waystone activated yet: use one to activate it.");
            return;
        }
        self.native_chat("Waystones (stand by one and type /waystones <number>):");
        for (i, w) in stones.iter().enumerate() {
            let [x, y, z] = w.pos;
            let dim = w.dimension.trim_start_matches("minecraft:");
            self.native_chat(&format!("{}. {x}, {y}, {z} ({dim})", i + 1));
        }
    }

    // `use_hand`'s HEAD: a waystone takes the use as its block would.
    pub(super) fn waystones_use(&mut self, hit: Option<&Hit>) -> ControlFlow<Result<bool>> {
        // shortcut: single player only; on servers and LAN guests the stone does nothing yet.
        if !self.native_on(ID) || self.remote.is_some() || self.world_root.as_os_str().is_empty() {
            return ControlFlow::Continue(());
        }
        let Some(hit) = hit else {
            return ControlFlow::Continue(());
        };
        // As vanilla: sneaking with something in hand skips the block.
        let sneaking = self.keys.contains(&KeyCode::ShiftLeft);
        let hands_full = self.stack_in(self.selected).is_some() || self.offhand.is_some();
        if sneaking && hands_full {
            return ControlFlow::Continue(());
        }
        let [x, y, z] = hit.pos;
        let state = self.world.block(x, y, z);
        let Some(s) = self.pack.assets.registry.state(state) else {
            return ControlFlow::Continue(());
        };
        if !is_waystone(&s.name) {
            return ControlFlow::Continue(());
        }
        let upper = s.properties.get("half").is_some_and(|h| &**h == "upper");
        let facing = s.properties.get("facing").map_or_else(|| "east".to_owned(), |f| f.to_string());
        let stone = Waystone {
            dimension: self.dim.id().into(),
            pos: [x, if upper { y - 1 } else { y }, z],
            facing,
        };
        let mut stones = self.waystones_load();
        if engine::activate(&mut stones, stone) {
            self.waystones_save(&stones);
            self.native_chat("Waystone activated.");
        }
        self.waystones_list(&stones);
        self.swing_arm();
        ControlFlow::Break(Ok(true))
    }

    pub(super) fn waystones_command(&mut self, command: &str, ran: &mut bool) {
        let mut words = command.split_whitespace();
        if *ran || !self.native_on(ID) || words.next() != Some("waystones") {
            return;
        }
        *ran = true;
        if self.remote.is_some() || self.world_root.as_os_str().is_empty() {
            self.native_chat("Waystones works in single player only.");
            return;
        }
        let stones = self.waystones_load();
        let Some(n) = words.next() else {
            self.waystones_list(&stones);
            return;
        };
        let Some(to) = n.parse::<usize>().ok().and_then(|n| stones.get(n.wrapping_sub(1))) else {
            self.native_chat(&format!("No waystone {n}: /waystones lists them."));
            return;
        };
        let Some(p) = self.native_player() else {
            return;
        };
        let here = self.dim.id();
        let near = stones.iter().any(|w| {
            let [x, y, z] = w.pos.map(|v| f64::from(v) + 0.5);
            w.dimension == here
                && (p.x - x).abs() <= REACH
                && (p.y - y).abs() <= REACH
                && (p.z - z).abs() <= REACH
        });
        if !near {
            self.native_chat("Stand by an activated waystone to travel.");
            return;
        }
        let [x, y, z] = engine::arrival(to.pos, &to.facing);
        let tp = format!("execute in {} run tp @s {x} {y} {z}", to.dimension);
        if !self.run_command_quietly(&tp) {
            self.native_chat("The waystone could not take you there.");
        }
    }
}
