use tiled::{Map, Object};

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
}

pub struct GameItem {
    pub position: [f32; 2],
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

pub struct GameMap {
    tiles: GameTiles,
    food_spawners: Vec<FoodSpawner>,

    items: Vec<GameItem>,
}

impl GameMap {
    fn load(map: Map) -> Self {
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
                food_spawners.push(FoodSpawner::new([x, y], [width, height]));
            }
        }

        // TODO: Remove me, testing
        let food = GameItem {
            position: [1500.0 / 128.0, 5300.0 / 128.0]
        };

        GameMap {
            tiles: tiles,
            food_spawners: food_spawners,
            items: vec!(food),
        }
    }

    pub fn tiles(&self) -> &GameTiles {
        &self.tiles
    }

    pub fn items(&self) -> &Vec<GameItem> {
        &self.items
    }

    pub fn update(&mut self, delta: f32) {
        // Make items fall down
        for item in &mut self.items {
            // Get the new position for the item
            let mut pos = item.position;
            pos[1] -= 0.5 * delta;

            // Make sure the new position doesn't collide with any tiles
            if self.tiles.get(pos[0] as u32, (pos[1] - 0.1) as u32)
                .map(|v| v.id)
                .unwrap_or(0) != 0 {
                continue;
            }

            // Apply the change
            item.position = pos;
        }
    }
}

pub struct GameModel {
    should_close: bool,
    map: GameMap,
}

impl GameModel {
    pub fn new(map: Map) -> Self {
        GameModel {
            should_close: false,
            map: GameMap::load(map),
        }
    }

    pub fn update(&mut self, delta: f32) {
        // Update the map data
        self.map.update(delta);
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn keep_running(&self) -> bool {
        !self.should_close
    }

    pub fn map(&self) -> &GameMap {
        &self.map
    }
}
