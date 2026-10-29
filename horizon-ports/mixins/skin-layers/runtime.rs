// 3D Skin Layers for Horizon, in the library's mob renderer: a player model
// draws its skin's outer layer again as voxels, posed like the body (as
// armour layers are). The game hands in each skin's pixels (`provide`)
// the first time a model with it is drawn (`wanted`).
use super::super::{Mob, MobData, Part, Polygon};
use super::skin_layers_engine::voxels;
use super::{Atlas, avatar, body, render_with};
use crate::mesher::Vertex;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    sync::Arc,
};

type Key = (String, bool);

thread_local! {
    // Each skin's voxels; None for a skin without usable pixels.
    static PARTS: RefCell<HashMap<Key, Option<Arc<Part>>>> = RefCell::new(HashMap::new());
    static WANTED: RefCell<HashSet<Key>> = RefCell::new(HashSet::new());
    // Set while the model with the voxels is drawn underneath.
    static DRAWING: Cell<bool> = const { Cell::new(false) };
}

// The skins (texture and slim arms) drawn without voxels yet.
pub fn wanted() -> Vec<(String, bool)> {
    WANTED.with(|w| w.borrow_mut().drain().collect())
}

// A skin's 64x64 RGBA pixels, or None when the game has none for it.
pub fn provide(skin: String, slim: bool, rgba: Option<&[u8]>) {
    let part = rgba.filter(|p| p.len() == 64 * 64 * 4).map(|p| {
        let opaque = |u: u32, v: u32| p[((v * 64 + u) * 4 + 3) as usize] >= 128;
        let children = voxels(opaque, slim)
            .into_iter()
            .map(|(name, quads)| Part {
                name: name.into(),
                pose: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                polygons: quads
                    .into_iter()
                    .map(|(normal, vertices)| Polygon { normal, vertices })
                    .collect(),
                children: Vec::new(),
            })
            .collect();
        Arc::new(Part {
            name: "skin_layers".into(),
            pose: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            polygons: Vec::new(),
            children,
        })
    });
    PARTS.with(|p| p.borrow_mut().insert((skin, slim), part));
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
    if DRAWING.with(Cell::get) || !avatar(m) {
        return core::ops::ControlFlow::Continue(());
    }
    let key = (body(m).1, m.slim);
    let part = PARTS.with(|p| p.borrow().get(&key).cloned());
    let Some(part) = part else {
        WANTED.with(|w| w.borrow_mut().insert(key));
        return core::ops::ControlFlow::Continue(());
    };
    let (Some(part), Some(uv)) = (part, atlas(&key.0)) else {
        return core::ops::ControlFlow::Continue(());
    };
    let mut layers = extra.to_vec();
    layers.push((&*part, uv, 0xFFFF_FFFF));
    DRAWING.with(|d| d.set(true));
    render_with(m, alpha, camera, atlas, light, data, &layers, out);
    DRAWING.with(|d| d.set(false));
    core::ops::ControlFlow::Break(())
}
