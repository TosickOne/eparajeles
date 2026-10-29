# Horizon ports

Native Horizon mods ([horizon-mixins](https://github.com/horizon-rust/horizon-mixins)) that bring
the behaviour of Java mods to [Horizon](https://github.com/horizon-rust/horizon). Each one is
written from the original mod's public description, never from its code. They are for local
use; publishing a port needs permission from the original mod's author.

Every bundle in `mixins/` is pinned to Horizon `c9d61dc` (2026-10-29), so they all build together.

| Port | Java original | What it does | State |
|---|---|---|---|
| `mixins/sorted` | Sorted | Quick Stack: **G** (rebindable in Options > Mods) in an open chest, barrel, ender chest or shulker box moves the main inventory's stacks of items the container already holds into it. Sorting is Horizon's own extra (Options > Extras). | Written, not built against Horizon yet |
| `mixins/world-map` | Xaero's World Map | **M** (rebindable) opens a big north-up map over the HUD of every chunk you have seen in this world and dimension; the wheel zooms. Saved every 30 s under `horizon-mods/.horizon-mod-data/world_map/<world>/`. On servers it lasts for the session only. No panning, waypoints or cave mode yet; the Nether shows its floor below y 100. | Written, not built against Horizon yet |
| `mixins/waystones` | Waystones | Single player: using a waystone block (the blocks come from the Waystones jar, carried over by the launcher's mod browser) activates it and lists the activated ones in chat; `/waystones <n>` while standing by an activated waystone travels to the n-th, in front of it, in any dimension. No menu, names, XP cost, scrolls, sharestones or warp plates yet. | Written, not built against Horizon yet |
| `mixins/sounds` | Sounds | With the Sounds jar's assets in the game (carried over by the mod browser), every vanilla block the mod has sounds for uses them to break, step, be placed, hit and land on. Its 120 replacements of vanilla sound files work as a resource pack: put the jar in `resourcepacks/`. Its UI, item and container sounds (opening screens, picking up items...) and its shared `*_object` sets are not wired up yet. | Written, not built against Horizon yet |
| `mixins/presence-footsteps` | Presence Footsteps | With the jar's assets in the game (mod browser), your own steps play the mod's acoustics for the block underfoot (its `blockmap.json` by block id, else `primitivemap.json` by the block's step sound), walk or run. Other players' and mobs' steps, landing and jumping sounds, block tags and sub-states in the blockmap, delayed parts and the mod's volume settings are not ported yet. | Written, not built against Horizon yet |

## Build and play

In the Horizon launcher: Settings → Game → **Native mods (experimental)** on, then from a terminal:

```sh
horizon-launcher native-build path/to/horizon-ports/mixins --trust-native-code
```

That builds the game with every bundle in `mixins/` (about 10 minutes the first time). To try one
port alone, pass its folder (`mixins/sorted`). Native mods are not sandboxed: they run with your
user's rights.

## Tests

The plain logic of each port lives in its `engine.rs` and is tested without Horizon:

```sh
cargo test
```
