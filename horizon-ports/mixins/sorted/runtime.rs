// Sorted's Quick Stack for Horizon: a key moves every main-inventory stack
// whose item the open chest already holds into it. Sorting itself is
// Horizon's own extra (Options > Extras > Sorting).
use super::native_hooks::NativeMod;
use super::{Game, Screen};
use horizon_client::{containers::BlockEntity, menu::Input};

pub(super) const ID: &str = "sorted";
const KEY: &str = "quick_stack";
const DEFAULT_KEY: &str = "key.keyboard.g";

pub(super) fn listing() -> NativeMod {
    NativeMod {
        id: ID.into(),
        title: "Sorted (Quick Stack) 0.1.0".into(),
        description: "Moves the stacks of items the open chest already holds into it.".into(),
        keys: vec![[KEY.into(), "Quick Stack".into(), DEFAULT_KEY.into()]],
    }
}

impl Game<'_> {
    pub(super) fn sorted_key(&mut self, key: &str, pressed: bool) {
        if !pressed || !self.native_on(ID) || key != self.native_mod_key(ID, KEY, DEFAULT_KEY) {
            return;
        }
        let Some(container) = self.sorted_container() else {
            return;
        };
        // Clicks with an item in hand would drop or swap it.
        if self.carried.is_some() {
            return;
        }
        // The hotbar stays, as in Sorted.
        let player: Vec<usize> = (9..36).collect();
        let mine: Vec<_> = player.iter().map(|&s| self.slot_view(s)).collect();
        let theirs: Vec<_> = container.iter().map(|&s| self.slot_view(s)).collect();
        let moves = super::sorted_engine::quick_stack(&mine, &theirs, |a, b| {
            a.same_item_same_components(b)
        });
        for i in moves {
            self.tweak_click(player[i], 0, Input::QuickMove);
        }
    }

    // The open chest-like container's slots: the screens Horizon's sort
    // buttons show on (chests, barrels, ender chests, shulker boxes).
    fn sorted_container(&self) -> Option<Vec<usize>> {
        if self.screen != Screen::Inventory
            || self.station.is_some()
            || self.mount_active()
            || self.recipe_search_focused()
        {
            return None;
        }
        let chest_like = if self.remote.is_some() {
            self.remote_storage_open()
        } else {
            matches!(self.container_kind(), Some(BlockEntity::Chest(_)))
                && (self.chest_rows().is_some() || self.container_block_is_shulker())
        };
        let slots: Vec<usize> = self
            .slots()
            .into_iter()
            .filter(|s| (300..500).contains(s))
            .collect();
        (chest_like && !slots.is_empty()).then_some(slots)
    }
}
