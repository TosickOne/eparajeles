// 3D Skin Layers' voxels, written from its description: every opaque texel
// of a skin's outer layer (hat, jacket, sleeves, trousers) becomes a small
// cube standing out of the model instead of a flat sheet. Faces follow
// vanilla's box UV layout (`ModelPart.Cube`). Plain types only, tested
// without Horizon.

// One quad: outward normal and four (x, y, z, u, v) corners, in model
// pixels and 0..1 skin UV.
pub type Quad = ([f32; 3], [[f32; 5]; 4]);

// A model part's outer-layer box: its name, corner, size and texture origin.
struct Layer {
    part: &'static str,
    from: [f32; 3],
    size: [f32; 3],
    uv: [u32; 2],
}

fn layers(slim: bool) -> [Layer; 6] {
    let arm = if slim { 3.0 } else { 4.0 };
    [
        Layer { part: "head", from: [-4.0, -8.0, -4.0], size: [8.0, 8.0, 8.0], uv: [32, 0] },
        Layer { part: "body", from: [-4.0, 0.0, -2.0], size: [8.0, 12.0, 4.0], uv: [16, 32] },
        Layer { part: "right_arm", from: [if slim { -2.0 } else { -3.0 }, -2.0, -2.0], size: [arm, 12.0, 4.0], uv: [40, 32] },
        Layer { part: "left_arm", from: [-1.0, -2.0, -2.0], size: [arm, 12.0, 4.0], uv: [48, 48] },
        Layer { part: "right_leg", from: [-2.0, 0.0, -2.0], size: [4.0, 12.0, 4.0], uv: [0, 32] },
        Layer { part: "left_leg", from: [-2.0, 0.0, -2.0], size: [4.0, 12.0, 4.0], uv: [0, 48] },
    ]
}

// The six faces of a box: corner vertices (as ModelPart.Cube orders them),
// texture rect (u1, v1, u2, v2) and outward normal.
fn faces(from: [f32; 3], size: [f32; 3], uv: [u32; 2]) -> [([[f32; 3]; 4], [f32; 4], [f32; 3]); 6] {
    let [x0, y0, z0] = from;
    let [x1, y1, z1] = [x0 + size[0], y0 + size[1], z0 + size[2]];
    let v = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0],
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1],
    ];
    let [w, h, d] = size;
    let (u, t) = (uv[0] as f32, uv[1] as f32);
    let (f7, f8, f9, f10, f11, f12) = (u, u + d, u + d + w, u + d + w + w, u + d + w + d, u + d + w + d + w);
    let (f13, f14, f15) = (t, t + d, t + d + h);
    [
        ([v[5], v[4], v[0], v[1]], [f8, f13, f9, f14], [0.0, -1.0, 0.0]),
        ([v[2], v[3], v[7], v[6]], [f9, f14, f10, f13], [0.0, 1.0, 0.0]),
        ([v[0], v[4], v[7], v[3]], [f7, f14, f8, f15], [-1.0, 0.0, 0.0]),
        ([v[1], v[0], v[3], v[2]], [f8, f14, f9, f15], [0.0, 0.0, -1.0]),
        ([v[5], v[1], v[2], v[6]], [f9, f14, f11, f15], [1.0, 0.0, 0.0]),
        ([v[4], v[5], v[6], v[7]], [f11, f14, f12, f15], [0.0, 0.0, 1.0]),
    ]
}

// A one-pixel cube centred on `c`, every face showing texel (tu, tv).
fn cube(c: [f32; 3], tu: u32, tv: u32, out: &mut Vec<Quad>) {
    let (u, v) = ((tu as f32 + 0.5) / 64.0, (tv as f32 + 0.5) / 64.0);
    let h = 0.5;
    for axis in 0..3 {
        for sign in [-1.0f32, 1.0] {
            let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut corners = [[0.0f32; 5]; 4];
            let around = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
            for (k, (da, db)) in around.into_iter().enumerate() {
                let mut p = c;
                p[axis] += sign * h;
                // Counter-clockwise seen from outside on both signs.
                p[a] += da * h;
                p[b] += db * h * sign;
                corners[k] = [p[0], p[1], p[2], u, v];
            }
            let mut normal = [0.0; 3];
            normal[axis] = sign;
            out.push((normal, corners));
        }
    }
}

// The voxel quads of each part, for the skin whose texel (u, v) is opaque
// when `opaque(u, v)`.
pub fn voxels(opaque: impl Fn(u32, u32) -> bool, slim: bool) -> Vec<(&'static str, Vec<Quad>)> {
    layers(slim)
        .into_iter()
        .map(|l| {
            let mut quads = Vec::new();
            for (verts, [u1, v1, u2, v2], n) in faces(l.from, l.size, l.uv) {
                let (a, b, c) = (verts[1], verts[0], verts[2]);
                let (cols, rows) = ((u2 - u1).abs() as u32, (v2 - v1).abs() as u32);
                for i in 0..cols {
                    for j in 0..rows {
                        let (tu, tv) = (u1.min(u2) as u32 + i, v1.min(v2) as u32 + j);
                        if !opaque(tu, tv) {
                            continue;
                        }
                        let s = (tu as f32 + 0.5 - u1) / (u2 - u1);
                        let t = (tv as f32 + 0.5 - v1) / (v2 - v1);
                        let center = std::array::from_fn(|k| {
                            a[k] + (b[k] - a[k]) * s + (c[k] - a[k]) * t + n[k] * 0.5
                        });
                        cube(center, tu, tv, &mut quads);
                    }
                }
            }
            (l.part, quads)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_hat_is_one_voxel_per_texel_of_its_six_faces() {
        let parts = voxels(|u, v| v < 16 && u >= 32, false);
        let head = &parts.iter().find(|p| p.0 == "head").unwrap().1;
        // 8x8 on each of six faces, six quads a voxel.
        assert_eq!(head.len(), 6 * 64 * 6);
        assert!(parts.iter().filter(|p| p.0 != "head").all(|p| p.1.is_empty()));
    }

    #[test]
    fn voxels_stand_just_outside_their_face() {
        // One texel of the hat's front (the north face starts at u 40, v 8).
        let parts = voxels(|u, v| (u, v) == (40, 8), false);
        let head = &parts[0].1;
        assert_eq!(head.len(), 6);
        let zs: Vec<f32> = head.iter().flat_map(|q| q.1.iter().map(|c| c[2])).collect();
        let (lo, hi) = zs.iter().fold((f32::MAX, f32::MIN), |(l, h), &z| (l.min(z), h.max(z)));
        assert_eq!((lo, hi), (-5.0, -4.0));
    }

    #[test]
    fn slim_arms_are_narrower() {
        let wide = voxels(|u, v| (40..56).contains(&u) && (32..48).contains(&v), false);
        let slim = voxels(|u, v| (40..56).contains(&u) && (32..48).contains(&v), true);
        let count = |p: &Vec<(&str, Vec<Quad>)>| p.iter().find(|x| x.0 == "right_arm").unwrap().1.len();
        assert!(count(&slim) < count(&wide));
    }
}
