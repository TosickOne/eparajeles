// Xaero's World Map for Horizon: every chunk the player has seen, kept per
// world and dimension, on a big map that M opens over the HUD. The wheel
// zooms while it is open.
use super::native_hooks::NativeMod;
use super::{Dim, Game};
use horizon_client::{
    hud::HudBatch,
    minimap::{Layer, world_map::WorldMap},
};
use std::{
    cell::RefCell,
    path::PathBuf,
    time::{Duration, Instant},
};

pub(super) const ID: &str = "world_map";
const KEY: &str = "open";
const DEFAULT_KEY: &str = "key.keyboard.m";
const SCAN_BUDGET: Duration = Duration::from_millis(2);
const SAVE_EVERY: Duration = Duration::from_secs(30);
// GUI pixels per map cell, and the most cells a side draws.
const CELL: f32 = 2.0;
const MAX_CELLS: (usize, usize) = (240, 135);

struct State {
    // The world and dimension shown, and its file (none on a server).
    at: (PathBuf, Dim),
    file: Option<PathBuf>,
    map: WorldMap,
    saved: Instant,
    open: bool,
    blocks_per_cell: f32,
    // The composed cells and what they were composed for.
    cells: Vec<u32>,
    composed: Option<(i32, i32, u32, usize, usize)>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "World Map 0.1.0".into(),
        description: "A map of every chunk you have seen, saved per world.".into(),
        keys: vec![[KEY.into(), "Open the world map".into(), DEFAULT_KEY.into()]],
    }
}

fn layer(dim: Dim) -> Layer {
    // shortcut: the Nether shows its floor below y 100 and the Overworld its surface, no cave mode yet.
    if dim == Dim::Nether { Layer::Below(100) } else { Layer::Surface }
}

fn save(state: &State) {
    let Some(file) = &state.file else {
        return;
    };
    let tmp = file.with_extension("tmp");
    let written = file.parent().map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&tmp, state.map.to_bytes()))
        .and_then(|()| std::fs::rename(&tmp, file));
    if let Err(e) = written {
        eprintln!("[world_map] could not save {}: {e}", file.display());
    }
}

impl Game<'_> {
    // The map's file: one per world and dimension. Servers keep theirs in memory.
    fn world_map_file(&self, dim: Dim) -> Option<PathBuf> {
        if self.remote.is_some() || self.world_root.as_os_str().is_empty() {
            return None;
        }
        let world = self.world_root.file_name()?.to_string_lossy().into_owned();
        let dim = dim.id().replace(':', "_");
        Some(self.native_data_dir(ID).join(world).join(format!("{dim}.hxwm")))
    }

    pub(super) fn world_map_frame(&mut self) {
        if !self.native_on(ID) || !self.native_window() {
            return;
        }
        let Some(p) = self.native_player() else {
            STATE.with(|s| {
                if let Some(old) = s.borrow_mut().take() {
                    save(&old);
                }
            });
            return;
        };
        let at = (self.world_root.clone(), self.dim);
        let file = self.world_map_file(self.dim);
        let (bx, bz) = (p.x.floor() as i32, p.z.floor() as i32);
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            if s.as_ref().is_none_or(|s| s.at != at) {
                let old = s.take();
                if let Some(old) = &old {
                    save(old);
                }
                let map = file
                    .as_ref()
                    .and_then(|f| std::fs::read(f).ok())
                    .and_then(|b| WorldMap::from_bytes(&b, layer(at.1)))
                    .unwrap_or_default();
                *s = Some(State {
                    at: at.clone(),
                    file: file.clone(),
                    map,
                    saved: Instant::now(),
                    open: old.as_ref().is_some_and(|o| o.open),
                    blocks_per_cell: old.as_ref().map_or(2.0, |o| o.blocks_per_cell),
                    cells: Vec::new(),
                    composed: None,
                });
            }
            let s = s.as_mut().expect("set above");
            s.map.update(
                &self.world,
                self.res.facts(),
                (bx >> 4, bz >> 4),
                self.settings.render_distance.max(2),
                layer(at.1),
                SCAN_BUDGET,
            );
            if s.saved.elapsed() >= SAVE_EVERY {
                save(s);
                s.saved = Instant::now();
            }
        });
    }

    pub(super) fn world_map_key(&mut self, key: &str, pressed: bool) {
        if !pressed || !self.native_on(ID) || key != self.native_mod_key(ID, KEY, DEFAULT_KEY) {
            return;
        }
        // The key also types into chat and signs: it opens the map only while playing, and closes it anywhere.
        let playing = self.native_playing();
        STATE.with(|s| {
            if let Some(s) = s.borrow_mut().as_mut()
                && (playing || s.open)
            {
                s.open = !s.open;
            }
        });
    }

    pub(super) fn world_map_wheel(&mut self, steps: f32, claimed: &mut bool) {
        if *claimed || !self.native_on(ID) {
            return;
        }
        STATE.with(|s| {
            if let Some(s) = s.borrow_mut().as_mut()
                && s.open
            {
                let factor = if steps > 0.0 { 0.5 } else { 2.0 };
                s.blocks_per_cell = (s.blocks_per_cell * factor).clamp(0.5, 32.0);
                *claimed = true;
            }
        });
    }

    pub(super) fn world_map_hud(&mut self, batch: &mut HudBatch) {
        if !self.native_on(ID) {
            return;
        }
        let Some(p) = self.native_player() else {
            return;
        };
        let w = (((batch.width - 40.0) / CELL).max(16.0) as usize).min(MAX_CELLS.0);
        let h = (((batch.height - 60.0) / CELL).max(16.0) as usize).min(MAX_CELLS.1);
        let (left, top) = ((batch.width - w as f32 * CELL) / 2.0, 30.0);
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let Some(s) = s.as_mut().filter(|s| s.open) else {
                return;
            };
            let want = (
                (p.x / f64::from(s.blocks_per_cell)).floor() as i32,
                (p.z / f64::from(s.blocks_per_cell)).floor() as i32,
                s.blocks_per_cell.to_bits(),
                w,
                h,
            );
            if s.composed != Some(want) || s.map.changed() {
                s.cells = s.map.compose(p.x, p.z, s.blocks_per_cell, w, h);
                s.composed = Some(want);
            }
            self.native_fill(
                batch,
                [left - 2.0, top - 14.0, w as f32 * CELL + 4.0, h as f32 * CELL + 16.0],
                0xC0101010,
            );
            // One fill per run of equal cells in a row.
            for j in 0..h {
                let row = &s.cells[j * w..(j + 1) * w];
                let mut i = 0;
                while i < w {
                    let argb = row[i];
                    let run = row[i..].iter().take_while(|&&c| c == argb).count();
                    if argb != 0 {
                        let rect = [left + i as f32 * CELL, top + j as f32 * CELL, run as f32 * CELL, CELL];
                        self.native_fill(batch, rect, argb);
                    }
                    i += run;
                }
            }
            let (cx, cy) = (left + w as f32 * CELL / 2.0, top + h as f32 * CELL / 2.0);
            self.native_fill(batch, [cx - 2.0, cy - 2.0, 4.0, 4.0], 0xFFFFFFFF);
            let title = format!(
                "World map  {}, {}  ({} blocks/px; wheel zooms)",
                p.x.floor(),
                p.z.floor(),
                s.blocks_per_cell / CELL
            );
            self.native_text(batch, &title, [left, top - 11.0], 0xFFFFFF, true);
        });
    }
}
