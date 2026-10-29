// The game side of 3D Skin Layers: each frame, the pixels of the skins the
// renderer asked for, from the skin downloads or the GUI atlas (the
// built-in skins).
use super::Game;
use super::native_hooks::NativeMod;
use horizon_client::mobs::render::skin_layers;

pub(super) const ID: &str = "skin_layers";

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "3D Skin Layers 0.1.0".into(),
        description: "The outer layer of skins stands out in 3D.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn skin_layers_frame(&mut self) {
        let wanted = skin_layers::wanted();
        if wanted.is_empty() || !self.native_on(ID) {
            return;
        }
        let downloads = self.skin_downloads.borrow();
        let atlas = &self.gui.atlas;
        for (skin, slim) in wanted {
            let from_atlas = atlas.sprites.get(&skin).filter(|r| r.w == 64 && r.h == 64).map(|r| {
                let (x0, y0) = ((r.u0 * atlas.width as f32).round() as usize, (r.v0 * atlas.height as f32).round() as usize);
                let row = atlas.width as usize * 4;
                (0..64)
                    .flat_map(|y| {
                        let at = (y0 + y) * row + x0 * 4;
                        atlas.pixels.get(at..at + 64 * 4).unwrap_or(&[]).to_vec()
                    })
                    .collect::<Vec<u8>>()
            });
            let rgba = horizon_client::skins::skin_layers::image(&downloads, &skin)
                .map(<[u8]>::to_vec)
                .or(from_atlas);
            skin_layers::provide(skin, slim, rgba.as_deref());
        }
    }
}
