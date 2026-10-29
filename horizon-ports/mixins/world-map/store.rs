// The world map's memory: the minimap's tiles, kept after their chunks
// unload, composed into a map of any size and saved as bytes. It sits in
// `horizon_client::minimap` to reuse its scan and shading.
use super::{Layer, Palette, RESCAN, Tile, UNKNOWN, WALL, brightness, packed_rgba, scan_tile};
use crate::{blocks::BlockFacts, fast_hash::FxHashMap, world::World};
use std::time::{Duration, Instant};

const MAGIC: &[u8; 5] = b"HXWM1";
// x, z, then 256 colours, 256 heights (i16 LE) and 256 depths.
const TILE_BYTES: usize = 8 + 256 + 512 + 256;

#[derive(Default)]
pub struct WorldMap {
    tiles: FxHashMap<(i32, i32), Tile>,
    palette: Option<Palette>,
    // Tiles changed since the last compose.
    changed: bool,
}
impl WorldMap {
    // Scan the loaded chunks around chunk `center`, nearest first, within
    // `budget`; tiles stay after their chunks unload.
    pub fn update(
        &mut self,
        world: &World,
        facts: &BlockFacts,
        center: (i32, i32),
        radius: i32,
        layer: Layer,
        budget: Duration,
    ) {
        let palette = self.palette.get_or_insert_with(|| Palette::new(facts));
        let started = Instant::now();
        let air = world.air.0;
        for ring in 0..=radius {
            for (dx, dz) in super::ring_offsets(ring) {
                let key = (center.0 + dx, center.1 + dz);
                let Some(chunk) = world.chunks.get(&key) else {
                    continue;
                };
                let stale = match self.tiles.get(&key) {
                    None => true,
                    Some(t) if t.layer != layer => true,
                    Some(t) => t.revision != world.revision && t.scanned.elapsed() >= RESCAN,
                };
                if !stale {
                    continue;
                }
                let tile = scan_tile(chunk, palette, air, layer, world.revision);
                if self.tiles.get(&key).is_none_or(|t| {
                    t.color != tile.color || t.height != tile.height || t.depth != tile.depth
                }) {
                    self.changed = true;
                }
                self.tiles.insert(key, tile);
                if started.elapsed() >= budget {
                    return;
                }
            }
        }
    }
    // Whether tiles changed since the last `compose`.
    pub fn changed(&self) -> bool {
        self.changed
    }
    // A `w` x `h` map centred on (x, z) at `bpp` blocks per cell, north up:
    // ARGB per cell, row by row, 0 where nothing was seen.
    pub fn compose(&mut self, x: f64, z: f64, bpp: f32, w: usize, h: usize) -> Vec<u32> {
        self.changed = false;
        let bpp = f64::from(bpp);
        let column = |x: i32, z: i32| -> Option<(u8, i16, u8)> {
            let t = self.tiles.get(&(x >> 4, z >> 4))?;
            let i = ((z & 15) * 16 + (x & 15)) as usize;
            Some((t.color[i], t.height[i], t.depth[i]))
        };
        let mut out = Vec::with_capacity(w * h);
        for j in 0..h {
            for i in 0..w {
                let wx = x + bpp * (i as f64 + 0.5 - w as f64 / 2.0);
                let wz = z + bpp * (j as f64 + 0.5 - h as f64 / 2.0);
                let (bx, bz) = (wx.floor() as i32, wz.floor() as i32);
                let [r, g, b, a] = match column(bx, bz) {
                    Some((color, height, depth)) if color & !WALL != 0 => {
                        let north = column(bx, bz - 1).filter(|c| c.0 & !WALL != 0).map(|c| c.1);
                        packed_rgba(color & !WALL, brightness(color, height, north, depth, bx + bz))
                    }
                    _ => UNKNOWN,
                };
                let seen = column(bx, bz).is_some();
                out.push(if seen { u32::from_be_bytes([a, r, g, b]) } else { 0 });
            }
        }
        out
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(MAGIC.len() + self.tiles.len() * TILE_BYTES);
        out.extend_from_slice(MAGIC);
        let mut keys: Vec<_> = self.tiles.keys().copied().collect();
        keys.sort_unstable();
        for key in keys {
            let t = &self.tiles[&key];
            out.extend_from_slice(&key.0.to_le_bytes());
            out.extend_from_slice(&key.1.to_le_bytes());
            out.extend_from_slice(&t.color);
            for h in t.height {
                out.extend_from_slice(&h.to_le_bytes());
            }
            out.extend_from_slice(&t.depth);
        }
        out
    }
    // A saved map, its tiles marked as scanned in `layer`; None for bytes
    // that are not one (a broken or foreign file).
    pub fn from_bytes(bytes: &[u8], layer: Layer) -> Option<Self> {
        let body = bytes.strip_prefix(MAGIC.as_slice())?;
        if body.len() % TILE_BYTES != 0 {
            return None;
        }
        let mut map = Self::default();
        for rec in body.chunks_exact(TILE_BYTES) {
            let int = |at: usize| i32::from_le_bytes(rec[at..at + 4].try_into().unwrap());
            let mut tile = Tile {
                color: rec[8..264].try_into().unwrap(),
                height: [0; 256],
                depth: rec[776..1032].try_into().unwrap(),
                // Revision 0: rescanned once its chunk loads again.
                revision: 0,
                layer,
                scanned: Instant::now(),
            };
            for (i, h) in rec[264..776].chunks_exact(2).enumerate() {
                tile.height[i] = i16::from_le_bytes([h[0], h[1]]);
            }
            map.tiles.insert((int(0), int(4)), tile);
        }
        map.changed = true;
        Some(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_round_trip_and_reject_broken_files() {
        let mut map = WorldMap::default();
        let mut tile = Tile {
            color: [0; 256],
            height: [-64; 256],
            depth: [0; 256],
            revision: 3,
            layer: Layer::Surface,
            scanned: Instant::now(),
        };
        tile.color[5] = 1;
        tile.height[5] = 70;
        tile.depth[7] = 4;
        map.tiles.insert((-3, 9), tile);
        let bytes = map.to_bytes();
        let back = WorldMap::from_bytes(&bytes, Layer::Surface).unwrap();
        let t = &back.tiles[&(-3, 9)];
        assert_eq!((t.color[5], t.height[5], t.depth[7]), (1, 70, 4));
        assert!(WorldMap::from_bytes(&bytes[..bytes.len() - 1], Layer::Surface).is_none());
        assert!(WorldMap::from_bytes(b"junk", Layer::Surface).is_none());
    }

    #[test]
    fn unseen_cells_are_transparent() {
        let mut map = WorldMap::default();
        assert_eq!(map.compose(0.0, 0.0, 1.0, 2, 2), [0; 4]);
    }
}
