use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use vespera_content::ItemId;

pub const PACK_SLOTS: usize = 28;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u32,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Pack {
    slots: [Option<ItemStack>; PACK_SLOTS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the pack has no space for the items")]
pub struct PackFull;

impl Pack {
    pub fn add(&mut self, item: &ItemId, count: u32, stack_size: u32) -> Result<(), PackFull> {
        if self.space_for(item, stack_size) < u64::from(count) {
            return Err(PackFull);
        }
        let mut remaining = count;
        for stack in self
            .slots
            .iter_mut()
            .flatten()
            .filter(|stack| &stack.item == item)
        {
            let added = remaining.min(stack_size.saturating_sub(stack.count));
            stack.count += added;
            remaining -= added;
        }
        for slot in self.slots.iter_mut().filter(|slot| slot.is_none()) {
            if remaining == 0 {
                break;
            }
            let added = remaining.min(stack_size);
            *slot = Some(ItemStack {
                item: item.clone(),
                count: added,
            });
            remaining -= added;
        }
        Ok(())
    }

    #[must_use]
    pub fn count(&self, item: &ItemId) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|stack| &stack.item == item)
            .map(|stack| stack.count)
            .sum()
    }

    #[must_use]
    pub fn free_slots(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_none()).count()
    }

    fn space_for(&self, item: &ItemId, stack_size: u32) -> u64 {
        self.slots
            .iter()
            .map(|slot| match slot {
                None => u64::from(stack_size),
                Some(stack) if &stack.item == item => {
                    u64::from(stack_size.saturating_sub(stack.count))
                }
                Some(_) => 0,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use vespera_content::ItemId;

    use super::{PACK_SLOTS, Pack, PackFull};

    fn logs() -> ItemId {
        ItemId::new("oak_logs")
    }

    fn feathers() -> ItemId {
        ItemId::new("feathers")
    }

    #[test]
    fn a_new_pack_has_28_free_slots() {
        assert_eq!(Pack::default().free_slots(), PACK_SLOTS);
        assert_eq!(PACK_SLOTS, 28);
    }

    #[test]
    fn items_that_do_not_stack_use_one_slot_each() {
        let mut pack = Pack::default();

        pack.add(&logs(), 3, 1).unwrap();

        assert_eq!(pack.count(&logs()), 3);
        assert_eq!(pack.free_slots(), 25);
    }

    #[test]
    fn stackable_items_fill_their_stack_first() {
        let mut pack = Pack::default();

        pack.add(&feathers(), 10, 100).unwrap();
        pack.add(&feathers(), 5, 100).unwrap();

        assert_eq!(pack.count(&feathers()), 15);
        assert_eq!(pack.free_slots(), 27);
    }

    #[test]
    fn a_full_stack_continues_in_a_new_slot() {
        let mut pack = Pack::default();

        pack.add(&feathers(), 150, 100).unwrap();

        assert_eq!(pack.count(&feathers()), 150);
        assert_eq!(pack.free_slots(), 26);
    }

    #[test]
    fn items_that_do_not_fit_change_nothing() {
        let mut pack = Pack::default();
        pack.add(&logs(), 27, 1).unwrap();
        let before = pack.clone();

        assert_eq!(pack.add(&logs(), 2, 1), Err(PackFull));
        assert_eq!(pack, before);
    }

    #[test]
    fn a_partial_stack_counts_as_free_space() {
        let mut pack = Pack::default();
        pack.add(&logs(), 27, 1).unwrap();
        pack.add(&feathers(), 90, 100).unwrap();

        assert_eq!(pack.add(&feathers(), 10, 100), Ok(()));
        assert_eq!(pack.add(&feathers(), 1, 100), Err(PackFull));
    }

    #[test]
    fn a_missing_item_has_a_count_of_zero() {
        assert_eq!(Pack::default().count(&logs()), 0);
    }

    #[test]
    fn adding_no_items_changes_nothing() {
        let mut pack = Pack::default();

        assert_eq!(pack.add(&logs(), 0, 1), Ok(()));
        assert_eq!(pack, Pack::default());
    }
}
