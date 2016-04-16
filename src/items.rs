use tiles::Tiles;

pub struct Item {
    pub position: [f32; 2],
    pub lifetime: f32,
}

pub struct Items {
    items: Vec<Option<Item>>,
}

impl Items {
    pub fn new() -> Self {
        Items {
            items: Vec::new(),
        }
    }

    pub fn for_each<F: FnMut(&Item)>(&self, mut f: F) {
        for item in &self.items {
            if let &Some(ref item) = item {
                f(item);
            }
        }
    }

    pub fn for_each_mut<F: FnMut(&mut Item)>(&mut self, mut f: F) {
        for item in &mut self.items {
            if let &mut Some(ref mut item) = item {
                f(item);
            }
        }
    }

    pub fn remove<F: Fn(&Item) -> bool>(&mut self, f: F) {
        for item in &mut self.items {
            let mut kill = false;
            if let &mut Some(ref mut item) = item {
                kill = f(item);
            }
            if kill {
                *item = None;
            }
        }
    }

    pub fn add(&mut self, item: Item) {
        // Find an empty slot
        for item_o in &mut self.items {
            if item_o.is_some() {
                continue;
            }

            // Found a slot, add it and return
            *item_o =  Some(item);
            return;
        }

        // Couldn't find one, add to end
        self.items.push(Some(item));
    }

    pub fn update(&mut self, tiles: &Tiles, delta: f32) {
        self.for_each_mut(|item| {
            // == Make them fall down ==

            // Get the new position for the item
            let mut pos = item.position;
            pos[1] -= 1.5 * delta;

            // Make sure the new position doesn't collide with any tiles
            if tiles.get(pos[0] as u32, (pos[1] - 0.1) as u32)
                .map(|v| v.id)
                .unwrap_or(0) != 0 {
                // Do nothing
            } else {
                // Apply the change
                item.position = pos;
            }

            // == Update their lifetime ==
            item.lifetime -= delta;
        });

        // Remove all items that have a lifetime of or less than zero
        self.remove(|item| item.lifetime <= 0.0);
    }
}
