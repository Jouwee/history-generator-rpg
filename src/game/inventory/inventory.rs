use crate::world::item::Item;

use super::inventory_container::InventoryContainer;

#[derive(Clone)]
pub(crate) struct Inventory {
    container: InventoryContainer,
    equipped: Option<Item>
}

impl Inventory {
    pub(crate) fn new() -> Inventory {
        Inventory { 
            container: InventoryContainer::new(35),
            equipped: None
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
        if let Some(item) = self.equipped.take() {
            items.push(item);
        }
        return items;
    }

    pub(crate) fn equip(&mut self, item: Item) {
        self.equipped = Some(item);
    }

    pub(crate) fn unequip(&mut self) -> Option<Item> {
        self.equipped.take()
    }

    pub(crate) fn equipped(&self) -> Option<&Item> {
        if let Some(equipped) = &self.equipped {
            return Some(equipped);
        }
        return None
    }

}