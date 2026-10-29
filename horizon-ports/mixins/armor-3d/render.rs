fn render(
    m: &Mob,
    alpha: f32,
    camera: glam::DVec3,
    atlas: Atlas,
    light: u32,
    data: &MobData,
    extra: &[(&Part, [f32; 4], u32)],
    out: &mut Vec<Vertex>,
) -> core::ops::ControlFlow<()> {
    crate::mobs::render::armor_3d::render(m, alpha, camera, atlas, light, data, extra, out)
}
