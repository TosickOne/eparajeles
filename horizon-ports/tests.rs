// The ports' engine.rs files hold their plain-Rust logic; this crate runs their tests
// without Horizon. Each one is also injected into the game as `crate::game::<id>_engine`.
#[path = "mixins/sorted/engine.rs"]
pub mod sorted;
#[path = "mixins/waystones/engine.rs"]
pub mod waystones;
#[path = "mixins/sounds/engine.rs"]
pub mod sounds;
#[path = "mixins/presence-footsteps/engine.rs"]
pub mod presence_footsteps;
#[path = "mixins/ambient-sounds/engine.rs"]
pub mod ambient_sounds;
#[path = "mixins/camera-overhaul/engine.rs"]
pub mod camera_overhaul;
#[path = "mixins/spawn-animations/engine.rs"]
pub mod spawn_animations;
#[path = "mixins/visuality/engine.rs"]
pub mod visuality;
#[path = "mixins/not-enough-animations/engine.rs"]
pub mod not_enough_animations;
#[path = "mixins/punchy/engine.rs"]
pub mod punchy;
#[path = "mixins/skin-layers/engine.rs"]
pub mod skin_layers;
