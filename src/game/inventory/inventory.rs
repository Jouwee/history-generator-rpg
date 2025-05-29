use std::slice::Iter;

use crate::world::item::Item;

use super::inventory_container::InventoryContainer;

#[derive(Clone)]
pub(crate) struct Inventory {
    container: InventoryContainer,
    equipped: Vec<(EquipmentType, Item)>
}

impl Inventory {
    pub(crate) fn new() -> Inventory {
        Inventory { 
            container: InventoryContainer::new(35),
            equipped: Vec::new()
        }
    }

    pub(crate) fn container_len(&self) -> usize {
        return self.container.len();
    }

    pub(crate) fn add(&mut self, item: Item) -> Result<(), Item> {
        return self.container.add(item);
    }

    pub(crate) fn item(&self, index: usize) -> &Option<Item> {
        return self.container.item(index);
    }

    pub(crate) fn item_mut(&mut self, index: usize) -> &mut Option<Item> {
        return self.container.item_mut(index);
    }

    pub(crate) fn take_all(&mut self) -> Vec<Item> {
        let mut items = self.container.take_all();
        for (_, item) in self.equipped.drain(..) {
            items.push(item);
        }
        return items;
    }

    pub(crate) fn equip(&mut self, slot: &EquipmentType, item: Item) {
        if self.equipped(slot).is_none() {
            self.equipped.push((slot.clone(), item));
        }
    }

    pub(crate) fn unequip(&mut self, slot: &EquipmentType) -> Option<Item> {
        let i = self.equipped.iter().position(|(i_slot, _)| i_slot == slot);
        if let Some(i) = i {
            let (_, item) = self.equipped.remove(i);
            return Some(item)
        }
        return None;
    }

    pub(crate) fn equipped(&self, slot: &EquipmentType) -> Option<&Item> {
        for (i_slot, item) in self.equipped.iter() {
            if i_slot == slot {
                return Some(item);
            }
        }
        return None;
    }

    pub(crate) fn all_equipped(&self) -> Iter<(EquipmentType, Item)> {
        self.equipped.iter()
    }

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EquipmentType {
    Hand,
    TorsoGarment,
    TorsoInner,
    Legs,
    Feet,
}

// TODO: Unit tests equipped