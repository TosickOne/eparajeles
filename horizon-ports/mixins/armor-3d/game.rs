// The game side of 3D Armor: each frame, the pixels of the armour textures
// the renderer asked for, from the GUI atlas.
use super::Game;
use super::native_hooks::NativeMod;
use horizon_client::mobs::render::armor_3d;

pub(super) const ID: &str = "armor_3d";

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "3D Armor 0.1.0".into(),
        description: "Worn armour is drawn in 3D, voxel by voxel from its texture.".into(),
        keys: Vec::new(),
    }
}

impl Game<'_> {
    pub(super) fn armor_3d_frame(&mut self) {
        let wanted = armor_3d::wanted();
        if wanted.is_empty() || !self.native_on(ID) {
            return;
        }
        let atlas = &self.gui.atlas;
        let (aw, ah) = (atlas.width as f32, atlas.height as f32);
        for uv in wanted {
            let [x0, y0, x1, y1] = [uv[0] * aw, uv[1] * ah, uv[2] * aw, uv[3] * ah].map(|v| v.round() as usize);
            let (w, h) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
            let row = atlas.width as usize * 4;
            let rgba: Option<Vec<u8>> = (0..h)
                .map(|y| atlas.pixels.get((y0 + y) * row + x0 * 4..(y0 + y) * row + (x0 + w) * 4).map(<[u8]>::to_vec))
                .collect::<Option<Vec<_>>>()
                .map(|rows| rows.concat());
            armor_3d::provide(uv, rgba.map(|p| (p, [w as u32, h as u32])));
        }
    }
}
