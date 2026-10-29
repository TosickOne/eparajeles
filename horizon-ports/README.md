# Horizon ports

Native Horizon mods ([horizon-mixins](https://github.com/horizon-rust/horizon-mixins)) that bring
the behaviour of Java mods to [Horizon](https://github.com/horizon-rust/horizon). Each one is
written from the original mod's public description, never from its code. They are for local
use; publishing a port needs permission from the original mod's author.

Every bundle in `mixins/` is pinned to Horizon `c9d61dc` (2026-10-29), so they all build together.

| Port | Java original | What it does | State |
|---|---|---|---|
| `mixins/sorted` | Sorted | Quick Stack: **G** (rebindable in Options > Mods) in an open chest, barrel, ender chest or shulker box moves the main inventory's stacks of items the container already holds into it. Sorting is Horizon's own extra (Options > Extras). | Written, not built against Horizon yet |

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
