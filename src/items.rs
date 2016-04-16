use cgmath::Vector2;
use tiles::Tiles;

#[derive(Eq, PartialEq)]
pub enum ItemState {
    Static,
    Falling,
    Carried
}

pub struct Item {
    pub position: Vector2<f32>,
    pub lifetime: f32,
    pub state: ItemState,
    pub claimed: bool,
}

impl Item {
    pub fn position(&self) -> Vector2<f32> {
        self.position
    }
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

    /*pub fn get(&self, id: u32) -> Option<&Item> {
        self.items.get(id as usize).and_then(|v| v.as_ref())
    }*/

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Item> {
        self.items.get_mut(id as usize).and_then(|v| v.as_mut())
    }

    pub fn remove(&mut self, id: u32) {
        self.items[id as usize] = None;
        println!("Item {} was removed", id);
    }

    pub fn remove_if<F: Fn(&Item) -> bool>(&mut self, f: F) {
        for i in 0..self.items.len() {
            let item = &mut self.items[i];

            let mut kill = false;
            if let &mut Some(ref mut item) = item {
                kill = f(item);
            }
            if kill {
                println!("Item {} was removed", i);
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
            // Make the item fall down, but only if it's not carried
            if item.state != ItemState::Carried {
                // Get the new position for the item
                let mut pos = item.position;
                pos[1] -= 1.5 * delta;

                // Make sure the new position doesn't collide with any tiles
                if tiles.get(pos[0] as u32, (pos[1] - 0.1) as u32)
                    .map(|v| v.class())
                    .unwrap_or(0) != 0 {
                    // We collided, switch to static mode
                    item.state = ItemState::Static;
                } else {
                    // Apply the change
                    item.position = pos;
                    item.state = ItemState::Falling;
                }
            }

            // Update the item's lifetime, but only if it's not claimed, reset its lifetime otherwise
            if item.claimed {
                item.lifetime = 60.0;
            } else {
                item.lifetime -= delta;
            }
        });

        // Remove all items that have a lifetime of or less than zero
        self.remove_if(|item| item.lifetime <= 0.0);
    }

    pub fn claim_resource(&mut self) -> Option<u32> {
        // Find the first unclaimed non-falling resource
        // TODO: Important! Find the closest one
        for i in 0..self.items.len() {
            let item_o = &mut self.items[i];
            if item_o.is_none() {
                continue;
            }

            let item = item_o.as_mut().unwrap();
            if item.state != ItemState::Static || item.claimed {
                continue;
            }

            // Actually claim the item
            item.claimed = true;

            return Some(i as u32);
        }

        // We couldn't find anything
        None
    }
}
