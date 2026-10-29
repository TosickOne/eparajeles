// 3D Armor's voxels, written from its description: each opaque texel of an
// armour layer becomes a small cube on the face it is drawn on. Works from a
// model's own quads, so every armour model and resource pack fits. Plain
// types only, tested without Horizon.

// One quad: outward normal and four (x, y, z, u, v) corners, in model pixels
// and 0..1 texture UV.
pub type Quad = ([f32; 3], [[f32; 5]; 4]);

// How far a voxel stands out of its face, in model pixels.
const DEPTH: f32 = 1.0;

// A box centred on `c` with half-extents `h`, every face showing UV (u, v).
fn cube(c: [f32; 3], h: [f32; 3], uv: [f32; 2], out: &mut Vec<Quad>) {
    for axis in 0..3 {
        for sign in [-1.0f32, 1.0] {
            let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut corners = [[0.0f32; 5]; 4];
            for (k, (da, db)) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].into_iter().enumerate() {
                let mut p = c;
                p[axis] += sign * h[axis];
                // Counter-clockwise seen from outside, as vanilla's cubes are.
                p[a] += da * h[a];
                p[b] += db * h[b] * sign;
                corners[k] = [p[0], p[1], p[2], uv[0], uv[1]];
            }
            let mut normal = [0.0; 3];
            normal[axis] = sign;
            out.push((normal, corners));
        }
    }
}

// The voxels of `quads` for a `size` texel texture whose texel (u, v) is
// opaque when `opaque(u, v)`. Corners follow vanilla's order: the second
// has the first UV corner, the first and third step along u and v.
pub fn voxelize(quads: &[Quad], size: [u32; 2], opaque: impl Fn(u32, u32) -> bool) -> Vec<Quad> {
    let (tw, th) = (size[0] as f32, size[1] as f32);
    let mut out = Vec::new();
    for (n, v) in quads {
        let (a, b, c) = (v[1], v[0], v[2]);
        let (u1, u2) = (a[3] * tw, b[3] * tw);
        let (v1, v2) = (a[4] * th, c[4] * th);
        if (u2 - u1).abs() < 0.5 || (v2 - v1).abs() < 0.5 {
            continue;
        }
        let (cols, rows) = ((u2 - u1).abs().round() as u32, (v2 - v1).abs().round() as u32);
        let (u0, v0) = (u1.min(u2).round() as u32, v1.min(v2).round() as u32);
        // A texel's extent on the face, half on each side of its centre.
        let extent = |from: [f32; 5], to: [f32; 5], count: u32| {
            std::array::from_fn::<f32, 3, _>(|k| (to[k] - from[k]).abs() / count as f32 / 2.0)
        };
        let (eu, ev) = (extent(a, b, cols), extent(a, c, rows));
        let half: [f32; 3] = std::array::from_fn(|k| if n[k] != 0.0 { DEPTH / 2.0 } else { eu[k] + ev[k] });
        for i in 0..cols {
            for j in 0..rows {
                let (tu, tv) = (u0 + i, v0 + j);
                if tu >= size[0] || tv >= size[1] || !opaque(tu, tv) {
                    continue;
                }
                let s = (tu as f32 + 0.5 - u1) / (u2 - u1);
                let t = (tv as f32 + 0.5 - v1) / (v2 - v1);
                let center = std::array::from_fn(|k| {
                    a[k] + (b[k] - a[k]) * s + (c[k] - a[k]) * t + n[k] * DEPTH / 2.0
                });
                let uv = [(tu as f32 + 0.5) / tw, (tv as f32 + 0.5) / th];
                cube(center, half, uv, &mut out);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // The north face of a 2x2x2 box at the origin, UV (0..2, 0..2) of a 4x4 texture.
    fn north() -> Quad {
        let uv = |u: f32, v: f32| [u / 4.0, v / 4.0];
        let p = |x: f32, y: f32, [u, v]: [f32; 2]| [x, y, 0.0, u, v];
        (
            [0.0, 0.0, -1.0],
            [p(2.0, 0.0, uv(2.0, 0.0)), p(0.0, 0.0, uv(0.0, 0.0)), p(0.0, 2.0, uv(0.0, 2.0)), p(2.0, 2.0, uv(2.0, 2.0))],
        )
    }

    #[test]
    fn one_voxel_per_opaque_texel_standing_out_of_the_face() {
        let all = voxelize(&[north()], [4, 4], |_, _| true);
        assert_eq!(all.len(), 4 * 6);
        let one = voxelize(&[north()], [4, 4], |u, v| (u, v) == (1, 1));
        assert_eq!(one.len(), 6);
        let zs: Vec<f32> = one.iter().flat_map(|q| q.1.iter().map(|c| c[2])).collect();
        let (lo, hi) = zs.iter().fold((f32::MAX, f32::MIN), |(l, h), &z| (l.min(z), h.max(z)));
        assert_eq!((lo, hi), (-1.0, 0.0));
        let xs: Vec<f32> = one.iter().flat_map(|q| q.1.iter().map(|c| c[0])).collect();
        let (lo, hi) = xs.iter().fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        // Texel u 1 maps to x 1..2 (u runs with x on this face).
        assert_eq!((lo, hi), (1.0, 2.0));
    }

    #[test]
    fn transparent_and_degenerate_quads_give_nothing() {
        assert!(voxelize(&[north()], [4, 4], |_, _| false).is_empty());
        let (n, mut v) = north();
        for c in &mut v {
            c[3] = 0.0;
        }
        assert!(voxelize(&[(n, v)], [4, 4], |_, _| true).is_empty());
    }
}
