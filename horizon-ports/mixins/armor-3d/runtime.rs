// 3D Armor for Horizon, in the library's mob renderer: every armour layer
// (players, armour stands, mobs) is drawn as voxels of its texture instead
// of flat inflated boxes. The game hands in each texture's pixels
// (`provide`) the first time it is drawn (`wanted`).
use super::super::{Mob, MobData, Part, Polygon};
use super::armor_3d_engine::{Quad, voxelize};
use super::{Atlas, render_with};
use crate::mesher::Vertex;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    sync::Arc,
};

// A texture by its atlas rect.
type Rect = [u32; 4];

thread_local! {
    // Each texture's pixels and size in texels; None when the game has none.
    static PIXELS: RefCell<HashMap<Rect, Option<(Vec<u8>, [u32; 2])>>> = RefCell::new(HashMap::new());
    static WANTED: RefCell<HashSet<Rect>> = RefCell::new(HashSet::new());
    // Voxel models by the armour model's address and its texture.
    // shortcut: models are keyed by address, as mob data is loaded once per resource reload; a reload keeps stale entries.
    static VOXELS: RefCell<HashMap<(usize, Rect), Arc<Part>>> = RefCell::new(HashMap::new());
    static DRAWING: Cell<bool> = const { Cell::new(false) };
}

fn key(uv: [f32; 4]) -> Rect {
    uv.map(f32::to_bits)
}

// The atlas rects drawn without pixels yet, as (u0, v0, u1, v1).
pub fn wanted() -> Vec<[f32; 4]> {
    WANTED.with(|w| w.borrow_mut().drain().map(|r| r.map(f32::from_bits)).collect())
}

// The RGBA of the texture at atlas rect `uv`, `size` texels.
pub fn provide(uv: [f32; 4], rgba: Option<(Vec<u8>, [u32; 2])>) {
    let rgba = rgba.filter(|(p, s)| p.len() == (s[0] * s[1] * 4) as usize && s[0] > 0 && s[1] > 0);
    PIXELS.with(|p| p.borrow_mut().insert(key(uv), rgba));
}

fn voxel_part(part: &Part, rgba: &[u8], size: [u32; 2]) -> Part {
    let quads: Vec<Quad> = part.polygons.iter().map(|p| (p.normal, p.vertices)).collect();
    let opaque = |u: u32, v: u32| rgba[((v * size[0] + u) * 4 + 3) as usize] >= 128;
    Part {
        name: part.name.clone(),
        pose: part.pose,
        polygons: voxelize(&quads, size, opaque)
            .into_iter()
            .map(|(normal, vertices)| Polygon { normal, vertices })
            .collect(),
        children: part.children.iter().map(|c| voxel_part(c, rgba, size)).collect(),
    }
}

// The voxel model for `part` drawn with texture `uv`, when its pixels are in.
fn voxels(part: &Part, uv: [f32; 4]) -> Option<Arc<Part>> {
    let k = (part as *const Part as usize, key(uv));
    if let Some(v) = VOXELS.with(|v| v.borrow().get(&k).cloned()) {
        return Some(v);
    }
    let pixels = PIXELS.with(|p| p.borrow().get(&k.1).cloned());
    let Some(pixels) = pixels else {
        WANTED.with(|w| w.borrow_mut().insert(k.1));
        return None;
    };
    let (rgba, size) = pixels?;
    let v = Arc::new(voxel_part(part, &rgba, size));
    VOXELS.with(|m| m.borrow_mut().insert(k, v.clone()));
    Some(v)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    m: &Mob,
    alpha: f32,
    camera: glam::DVec3,
    atlas: Atlas,
    light: u32,
    data: &MobData,
    extra: &[(&Part, [f32; 4], u32)],
    out: &mut Vec<Vertex>,
) -> core::ops::ControlFlow<()> {
    if DRAWING.with(Cell::get) || extra.is_empty() {
        return core::ops::ControlFlow::Continue(());
    }
    // 3D Skin Layers' own voxels stay as they are.
    let built: Vec<Option<Arc<Part>>> = extra
        .iter()
        .map(|(part, uv, _)| (part.name != "skin_layers").then(|| voxels(part, *uv)).flatten())
        .collect();
    if built.iter().all(Option::is_none) {
        return core::ops::ControlFlow::Continue(());
    }
    let layers: Vec<(&Part, [f32; 4], u32)> = extra
        .iter()
        .zip(&built)
        .map(|(&(part, uv, color), v)| (v.as_deref().unwrap_or(part), uv, color))
        .collect();
    DRAWING.with(|d| d.set(true));
    render_with(m, alpha, camera, atlas, light, data, &layers, out);
    DRAWING.with(|d| d.set(false));
    core::ops::ControlFlow::Break(())
}
