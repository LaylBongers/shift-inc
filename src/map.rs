use tiled::{Map, Object};
use rand::{StdRng, Rng};

#[derive(Debug)]
pub struct GameTile {
    pub id: u32
}

impl GameTile {
    fn from_raw_id(raw_id: u32) -> Self {
        let id = if raw_id == 0 {
            1 // Default tile, solid flesh
        } else {
            raw_id - 1 // Actual tile
        };

        GameTile {
            id: id,
        }
    }
}

struct FoodSpawner {
    position: [f32; 2],
    size: [f32; 2],
}

impl FoodSpawner {
    fn new(position: [f32; 2], size: [f32; 2]) -> Self {
        FoodSpawner {
            position: position,
            size: size,
        }
    }

    fn spawn(&self, rng: &mut StdRng) -> GameItem {
        let x = rng.gen_range(self.position[0], self.position[0] + self.size[0]);
        let y = rng.gen_range(self.position[1], self.position[1] + self.size[1]);
        println!("Spawning food at {}, {}", x, y);

        GameItem {
            position: [x, y],
            lifetime: 20.0
        }
    }
}

pub struct GameItem {
    pub position: [f32; 2],
    pub lifetime: f32,
}

pub struct GameTiles {
    width: u32,
    height: u32,
    tiles: Vec<GameTile>,
}

impl GameTiles {
    fn load(map: &Map) -> Self {
        // Process the tiles
        let tiles_layer = map.layers.iter().find(|v| v.name == "Tiles").unwrap();
        let mut tiles = Vec::new();
        for row in &tiles_layer.tiles {
            for tile in row {
                tiles.push(GameTile::from_raw_id(*tile));
            }
        }

        GameTiles {
            width: map.width,
            height: map.height,
            tiles: tiles,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn position_for(&self, x: u32, y: u32) -> usize {
        let actual_y = self.height as usize - y as usize - 1; // Hacky flip, perhaps do this on loading
        x as usize + actual_y * self.width as usize
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&GameTile> {
        if y >= self.height || x >= self.width {
            None
        } else {
            // Grain is 1,0 -> 2,0 -> 0,1 x increases first
            let tile = self.tiles.get(self.position_for(x, y));
            assert!(tile.is_some(), "Tile at {},{} should have been some!", x, y);
            tile
        }
    }
}

pub struct GameItems {
    items: Vec<Option<GameItem>>,
}

impl GameItems {
    fn new() -> Self {
        GameItems {
            items: Vec::new(),
        }
    }

    pub fn for_each<F: FnMut(&GameItem)>(&self, mut f: F) {
        for item in &self.items {
            if let &Some(ref item) = item {
                f(item);
            }
        }
    }

    pub fn for_each_mut<F: FnMut(&mut GameItem)>(&mut self, mut f: F) {
        for item in &mut self.items {
            if let &mut Some(ref mut item) = item {
                f(item);
            }
        }
    }

    fn remove<F: Fn(&GameItem) -> bool>(&mut self, f: F) {
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

    fn add(&mut self, item: GameItem) {
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
}

pub struct GameMap {
    tiles: GameTiles,
    food_spawners: Vec<FoodSpawner>,

    items: GameItems,

    food_spawn_accum: f32,
}

impl GameMap {
    pub fn load(map: Map) -> Self {
        // Should have 1 layer and 1 object group
        assert_eq!(map.layers.len(), 1);
        assert_eq!(map.object_groups.len(), 1);

        // Load in the tiles
        let tiles = GameTiles::load(&map);

        // Process the food spawners
        let food_spawners_layer = map.object_groups.iter().find(|v| v.name == "Food Spawners").unwrap();
        let mut food_spawners = Vec::new();
        for obj in &food_spawners_layer.objects {
            if let &Object::Rect { x, y, width, height, visible: _ } = obj {
                let actual_height = height / 128.0;
                food_spawners.push(FoodSpawner::new(
                    [x / 128.0, tiles.height() as f32 - (y / 128.0) - actual_height],
                    [width / 128.0, actual_height]
                ));
            }
        }

        GameMap {
            tiles: tiles,
            food_spawners: food_spawners,

            items: GameItems::new(),

            food_spawn_accum: 0.0,
        }
    }

    pub fn tiles(&self) -> &GameTiles {
        &self.tiles
    }

    pub fn items(&self) -> &GameItems {
        &self.items
    }

    pub fn update(&mut self, delta: f32, rng: &mut StdRng) {
        // Update all items
        let tiles = &mut self.tiles;
        self.items.for_each_mut(|item| {
            // == Make them fall down ==

            // Get the new position for the item
            let mut pos = item.position;
            pos[1] -= 0.5 * delta;

            // Make sure the new position doesn't collide with any tiles
            if tiles.get(pos[0] as u32, (pos[1] - 0.1) as u32)
                .map(|v| v.id)
                .unwrap_or(0) != 0 {
                return;
            }

            // Apply the change
            item.position = pos;

            // == Update their lifetime ==
            item.lifetime -= delta;
        });

        // Remove all items that have a lifetime of or less than zero
        self.items.remove(|item| item.lifetime <= 0.0);

        // Spawn a food blob if time has passed and we don't already have 100
        self.food_spawn_accum += delta;
        while self.food_spawn_accum > 4.0 {
            self.food_spawn_accum -= 4.0;

            assert_eq!(self.food_spawners.len(), 1);
            let spawner = &self.food_spawners[0];
            let item = spawner.spawn(rng);
            self.items.add(item);
        }
    }
}
