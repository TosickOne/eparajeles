// A reduced Visuality for Horizon: ambient particles around the player,
// with vanilla's particle types: glints on amethyst, souls over soul sand,
// rain rings on open water.
use super::native_hooks::NativeMod;
use super::visuality_engine::{Effect, effect};
use super::Game;
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_core::position::BlockPos;
use std::cell::Cell;

pub(super) const ID: &str = "visuality";
// Blocks sampled per tick, within this many blocks of the player.
const SAMPLES: u32 = 48;
const RADIUS: i32 = 12;

thread_local! {
    static SEED: Cell<u64> = const { Cell::new(0x9E37_79B9_7F4A_7C15) };
}

// A small LCG: these particles do not need the game's random sources. 0..1.
fn rand() -> f64 {
    SEED.with(|s| {
        let v = s.get().wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        s.set(v);
        (v >> 11) as f64 / (1u64 << 53) as f64
    })
}

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Visuality 0.1.0".into(),
        description: "Glints on amethyst, souls over soul sand, rain rings on water.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn visuality_tick(&mut self) {
        if !self.native_on(ID) || !self.native_window() {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let center = [p.position.x.floor() as i32, p.position.y.floor() as i32, p.position.z.floor() as i32];
        let span = 2 * RADIUS + 1;
        let mut spawns = Vec::new();
        for _ in 0..SAMPLES {
            let offset = || (rand() * f64::from(span)) as i32 - RADIUS;
            let (x, y, z) = (center[0] + offset(), center[1] + offset(), center[2] + offset());
            let name = self.res.facts().get(self.world.block(x, y, z)).name.clone();
            let above = self.world.block(x, y + 1, z);
            let open = self.res.facts().is_air(above);
            let rain = open && self.raining_at(BlockPos { x, y: y + 1, z });
            if let Some((kind, chance)) = effect(&name, open, rain)
                && rand() < f64::from(chance)
            {
                spawns.push((kind, [x, y, z]));
            }
        }
        for (kind, [x, y, z]) in spawns {
            let (bx, by, bz) = (f64::from(x), f64::from(y), f64::from(z));
            let (ty, at, v) = match kind {
                Effect::Sparkle => (
                    ParticleType::ElectricSpark,
                    [bx + rand(), by + rand(), bz + rand()],
                    [0.0; 3],
                ),
                Effect::Soul => (ParticleType::Soul, [bx + rand(), by + 1.0, bz + rand()], [0.0, 0.03, 0.0]),
                Effect::RainRing => (ParticleType::Splash, [bx + rand(), by + 0.9, bz + rand()], [0.0; 3]),
            };
            self.particles.add(&ParticleOptions::simple(ty), at, v);
        }
    }
}
