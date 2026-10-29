// The activated waystones of a world, one per line ("<dimension> <x> <y> <z> <facing>"),
// in the order they were activated. Plain types only, tested without Horizon.

#[derive(Clone, Debug, PartialEq)]
pub struct Waystone {
    pub dimension: String,
    pub pos: [i32; 3],
    // The stone's `facing` property: travellers arrive on that side.
    pub facing: String,
}

pub fn parse(text: &str) -> Vec<Waystone> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let dimension = parts.next()?.to_owned();
            let mut pos = [0; 3];
            for p in &mut pos {
                *p = parts.next()?.parse().ok()?;
            }
            let facing = parts.next()?.to_owned();
            parts.next().is_none().then_some(Waystone { dimension, pos, facing })
        })
        .collect()
}

pub fn write(stones: &[Waystone]) -> String {
    stones
        .iter()
        .map(|w| format!("{} {} {} {} {}\n", w.dimension, w.pos[0], w.pos[1], w.pos[2], w.facing))
        .collect()
}

// Adds `stone` unless one stands at its place; true when it was new.
pub fn activate(stones: &mut Vec<Waystone>, stone: Waystone) -> bool {
    if stones.iter().any(|w| w.dimension == stone.dimension && w.pos == stone.pos) {
        return false;
    }
    stones.push(stone);
    true
}

// Where a player arrives: the block in front of the stone's `facing`, centred.
pub fn arrival(pos: [i32; 3], facing: &str) -> [f64; 3] {
    let [dx, dz] = match facing {
        "north" => [0, -1],
        "south" => [0, 1],
        "west" => [-1, 0],
        _ => [1, 0],
    };
    [
        f64::from(pos[0] + dx) + 0.5,
        f64::from(pos[1]),
        f64::from(pos[2] + dz) + 0.5,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_round_trip_and_skip_broken_ones() {
        let stones = vec![
            Waystone { dimension: "minecraft:overworld".into(), pos: [-10, 64, 3], facing: "north".into() },
            Waystone { dimension: "minecraft:the_nether".into(), pos: [5, 70, -8], facing: "east".into() },
        ];
        let text = write(&stones) + "garbage\nminecraft:overworld 1 2 3\n";
        assert_eq!(parse(&text), stones);
    }

    #[test]
    fn activating_twice_keeps_one() {
        let mut stones = Vec::new();
        let w = Waystone { dimension: "minecraft:overworld".into(), pos: [0, 64, 0], facing: "north".into() };
        assert!(activate(&mut stones, w.clone()));
        assert!(!activate(&mut stones, Waystone { facing: "south".into(), ..w }));
        assert_eq!(stones.len(), 1);
    }

    #[test]
    fn arrival_is_in_front_of_the_stone() {
        assert_eq!(arrival([0, 64, 0], "north"), [0.5, 64.0, -0.5]);
        assert_eq!(arrival([0, 64, 0], "up"), [1.5, 64.0, 0.5]);
    }
}
