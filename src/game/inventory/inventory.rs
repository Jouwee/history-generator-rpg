use crate::world::item::Item;

#[derive(Clone)]
pub struct Inventory {
    items: Vec<Item>,
    equipped: Option<usize>
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory { items: Vec::new(), equipped: None }
    }

    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn equip(&mut self, i: usize) {
        self.equipped = Some(i);
    }

    pub fn equipped(&self) -> Option<&Item> {
        match self.equipped {
            Some(i) => self.items.get(i),
            None => None
        }
    }

    pub fn len(&self) -> usize {
        return self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &Item, bool)> {
        self.items.iter().enumerate().map(|(i, it)| {
            let mut equip = false;
            if let Some(equipped) = self.equipped {
                equip = i == equipped;
            }
            (i, it, equip)
        })
    }

}