use crate::Item;

#[derive(Clone)]
pub(crate) struct InventoryContainer {
    items: Vec<Option<Item>>,
    none: Option<Item>
}

impl InventoryContainer {
    
    pub(crate) fn new(size: usize) -> Self {
        Self {
            items: vec![None; size],
            none: None
        }
    }

    pub(crate) fn len(&self) -> usize {
        return self.items.len();
    }

    pub(crate) fn add(&mut self, item: Item) -> Result<(), Item> {
        let i = self.items.iter().position(|item| item.is_none());
        if let Some(i) = i {
            self.items[i] = Some(item);
            return Result::Ok(())
        } else {
            return Result::Err(item)
        }
    }

    pub(crate) fn item(&self, index: usize) -> &Option<Item> {
        match self.items.get(index) {
            Some(v) => v,
            None => &None
        }
    }

    pub(crate) fn item_mut(&mut self, index: usize) -> &mut Option<Item> {
        match self.items.get_mut(index) {
            Some(v) => v,
            None => &mut self.none
        }
    }

    pub(crate) fn take(&mut self, index: usize) -> Option<Item> {
        if let Some(item) = self.items.get_mut(index) {
            return item.take()    
        }
        return None
    }

    pub(crate) fn take_all(&mut self) -> Vec<Item> {
        let mut result = Vec::new();
        for item in self.items.iter_mut() {
            if let Some(item) = item.take() {
                result.push(item);
            }
        }
        return result;
    }

}

#[cfg(test)]
mod tests_inventory_container {
    use crate::ItemFactory;

    use super::*;

    #[test]
    fn add() {
        let mut container = InventoryContainer::new(3);
        let item = ItemFactory::test();
        let r = container.add(item.clone());
        assert_eq!(r.is_ok(), true);
        let r = container.add(item.clone());
        assert_eq!(r.is_ok(), true);
        let r = container.add(item.clone());
        assert_eq!(r.is_ok(), true);
        let r = container.add(item.clone());
        assert_eq!(r.is_ok(), false);
    }

}