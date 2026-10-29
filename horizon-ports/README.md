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
| `mixins/ambient-sounds` | AmbientSounds | A reduced version: with the jar's assets in the game (mod browser), it reads the mod's regions (`basic/regions/*.json`: biomes, caves by sky light, rain, day or night, bad regions) and every 30 to 60 s plays one of the active regions' sounds around you. The mod's continuous loops and fades, air pockets, block features, heights, underwater, time curves and dimension files are not ported. | Written, not built against Horizon yet |
| `mixins/camera-overhaul` | CameraOverhaul | The camera rolls as you strafe and turn and pitches as you jump, fall and move forward, eased each frame, on top of the view bob. Explosion shake, idle sway and the mod's settings are not ported; the strengths are constants in `engine.rs`. | Written, not built against Horizon yet |
| `mixins/first-person` | First-person Model | In first person your own body (with its armour and held items) shows when you look down, drawn 0.35 blocks behind the eye so the head stays out of view. The first-person hand still draws as well, and the mod's settings and per-item hiding are not ported. | Written, not built against Horizon yet |
| `mixins/spawn-animations` | Spawn Animations | Single player: mobs that natural spawning adds climb out of the ground over their first second (drawn lower, easing up). Mobs from spawners, eggs and commands, the dust particles and the mod's settings are not ported. | Written, not built against Horizon yet |
| `mixins/visuality` | Visuality | A reduced version with vanilla's particle types (Horizon has no way to add new ones yet): glints on amethyst, souls rising from open soul sand and soil, splashes where rain hits open water, sampled around you each tick. The mod's own particle textures, hit particles (bones, feathers), slime blobs and settings are not ported. | Written, not built against Horizon yet |
| `mixins/not-enough-animations` | Not Enough Animations | Two of its third-person poses, for every player model (yours in F5, others on servers): the arm comes up to the mouth with a bite bob while eating or drinking, and both arms hold a filled map out in front. Its ladder, rowing, lantern, sleeping and other poses and its settings are not ported. | Written, not built against Horizon yet |
| `mixins/punchy` | Punchy! | In first person your main-hand arm shows under the item it holds, swinging with it as the empty arm does (not with drawn bows, crossbows, raised tridents and spears, or maps). Its tool-specific animations (the hoe and others) and settings are not ported. | Written, not built against Horizon yet |
| `mixins/skin-layers` | 3D Skin Layers | Every player model (yours in F5 and with First-person Model, others on servers) draws its skin's outer layer (hat, jacket, sleeves, trousers) again as one-pixel voxels standing out of the body, posed with it. Pixels come from downloaded skins or, for built-in skins, the GUI atlas. Each voxel is a full cube (no merged faces, no level of detail), and the mod's settings are not ported. | Written, not built against Horizon yet |
| `mixins/armor-3d` | 3D Armor | Every armour layer the renderer draws (players, armour stands, mobs; trims and dyes included) is drawn as voxels of its texture, built from the armour model's own faces, so resource packs and modded armour fit. Elytra and the horse armour are left flat; the mod's settings are not ported. | Written, not built against Horizon yet |

Not ported, on purpose:

- **Matcha Flavoured** has no code, its Fabric/NeoForge "mod" version included: that jar is the data pack and resource pack wrapped by Modrinth (`credits = 'Generated by Modrinth'`, no `.class` file). Install it from the launcher's mod browser; how much of it works is Horizon's data pack support (functions, predicates and item modifiers today; not yet its recipes, loot tables, advancements, villager trades or worldgen, which go under `data/minecraft/`).
- **Smooth Motion Blur** needs the previous frame's camera, which Horizon's shader API does not give yet; once it does, it is a Horizon shader pack rather than a native mod.
- **WaveyCapes** needs capes to draw first. Horizon downloads cape textures with the skins (`skins::lookup`) but keeps only the skin's location and has no cape model or renderer; vanilla's cape is the step before waving it.
- **Fancy Door Animations** needs doors drawn outside the chunk mesh while they swing. Horizon meshes doors with the terrain and redraws the section on each change, so an animated door needs a mesher hook that leaves the block out for the swing and an entity-pass door model in its place.

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
