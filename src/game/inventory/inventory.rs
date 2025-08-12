use crate::world::item::Item;

use super::inventory_container::InventoryContainer;

#[derive(Clone)]
pub(crate) struct Inventory {
    container: InventoryContainer,
    slots: [(EquipmentType, Option<Item>); 8]
}

impl Inventory {

    pub(crate) fn new() -> Inventory {
        Inventory { 
            container: InventoryContainer::new(35),
            slots: [
                (EquipmentType::TorsoInner, None),
                (EquipmentType::TorsoGarment, None),
                (EquipmentType::Legs, None),
                (EquipmentType::Head, None),
                (EquipmentType::Feet, None),
                (EquipmentType::Hand, None),
                (EquipmentType::Trinket, None),
                (EquipmentType::Trinket, None)
            ]
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
        for (_, item) in self.slots.iter_mut() {
            let item = item.take();
            if let Some(item) = item {
                items.push(item);
            }
        }
        return items;
    }

    pub(crate) fn equip(&mut self, slot: &EquipmentType, item: Item) {
        self.equip_i(slot, 0, item);
    }

    pub(crate) fn equipped(&self, slot: &EquipmentType) -> Option<&Item> {
        self.equipped_i(slot, 0)
    }

    pub(crate) fn equip_i(&mut self, slot_type: &EquipmentType, i: usize, item: Item) {
        let mut j = 0;
        for slot in self.slots.iter_mut() {
            if slot.0 == *slot_type {
                if j == i {
                    slot.1 = Some(item);
                    break;
                }
                j = j + 1;
            }
        }
    }

    pub(crate) fn unequip_i(&mut self, slot_type: &EquipmentType, i: usize) -> Option<Item> {
        let mut j = 0;
        for slot in self.slots.iter_mut() {
            if slot.0 == *slot_type {
                if j == i {
                    let item = slot.1.take();
                    slot.1 = None;
                    return item;
                }
                j = j + 1;
            }
        }
        return None;
    }

    pub(crate) fn equipped_i(&self, slot_type: &EquipmentType, i: usize) -> Option<&Item> {
        let mut j = 0;
        for slot in self.slots.iter() {
            if slot.0 == *slot_type {
                if j == i {
                    return slot.1.as_ref();
                }
                j = j + 1;
            }
        }
        return None;
    }

    pub(crate) fn all_equipped(&self) -> impl Iterator<Item = (&EquipmentType, &Item)> {
        self.slots.iter()
            .filter(|slot| slot.1.is_some())
            .map(|slot| (&slot.0, slot.1.as_ref().unwrap()))
    }

    pub(crate) fn auto_equip(&mut self) {
        for i in 0..self.container.len() {
            let mut equip_slot = None;
            if let Some(item) = self.container.item(i) {
                if let Some(equippable) = &item.equippable {
                    // TODO: Choose best
                    if self.equipped(&equippable.slot).is_none() {
                        equip_slot = Some(equippable.slot.clone());
                    }
                }
            }
            if let Some(slot) = equip_slot {
                if let Some(item) = self.container.take(i) {
                    self.equip(&slot, item);
                }
            }
        }
    }

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EquipmentType {
    Head,
    Hand,
    TorsoGarment,
    TorsoInner,
    Legs,
    Feet,
    Trinket,
}

#[cfg(test)]
mod tests_ai_groups {
    use crate::game::factory::item_factory::ItemFactory;

    use super::*;

    #[test]
    fn test_equip() {

        let item = ItemFactory::test();
        let mut inventory = Inventory::new();

        assert!(inventory.equipped(&EquipmentType::Hand).is_none());
        
        inventory.equip(&EquipmentType::Hand, item);
        assert!(inventory.equipped(&EquipmentType::Hand).is_some());

        let r = inventory.unequip_i(&EquipmentType::Hand, 0);
        assert!(r.is_some());
        assert!(inventory.equipped(&EquipmentType::Hand).is_none());

    }

}