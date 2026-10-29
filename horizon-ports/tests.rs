// The ports' engine.rs files hold their plain-Rust logic; this crate runs their tests
// without Horizon. Each one is also injected into the game as `crate::game::<id>_engine`.
#[path = "mixins/sorted/engine.rs"]
pub mod sorted;
#[path = "mixins/waystones/engine.rs"]
pub mod waystones;
