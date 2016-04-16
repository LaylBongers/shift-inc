use cgmath::Vector2;
use tiled::{Map, Object};
use rand::{StdRng, Rng};
use items::{Item, Items};

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

    fn spawn(&self, rng: &mut StdRng) -> Item {
        let x = rng.gen_range(self.position[0], self.position[0] + self.size[0]);
        let y = rng.gen_range(self.position[1], self.position[1] + self.size[1]);
        println!("Spawning food at {}, {}", x, y);

        Item {
            position: [x, y],
            lifetime: 30.0
        }
    }
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

    pub fn for_each<F: FnMut(u32, u32, &GameTile)>(&self, mut f: F) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let tile = self.get(x, y).unwrap();
                f(x, y, tile);
            }
        }
    }
}

pub struct Robot {
    position: Vector2<f32>
}

impl Robot {
    fn new(position: Vector2<f32>) -> Self {
        Robot {
            position: position
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }
}

pub struct Robots {
    robots: Vec<Robot>,
}

impl Robots {
    fn new() -> Self {
        Robots {
            robots: Vec::new(),
        }
    }

    fn add(&mut self, robot: Robot) {
        self.robots.push(robot);
    }

    pub fn for_each<F: FnMut(&Robot)>(&self, mut f: F) {
        for robot in &self.robots {
            //if let &Some(ref robot) = robot {
            f(robot);
            //}
        }
    }
}

pub struct GameMap {
    tiles: GameTiles,
    food_spawners: Vec<FoodSpawner>,

    items: Items,
    robots: Robots,

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

        // Find the core and walls in the map and spawn some robots for it
        let mut robots = Robots::new();
        tiles.for_each(|x, y, tile| {
            if tile.id != 2 && tile.id != 3 { return; }

            robots.add(Robot::new(Vector2::new(x as f32 + 0.5, y as f32 + 0.5)));
        });

        GameMap {
            tiles: tiles,
            food_spawners: food_spawners,

            items: Items::new(),
            robots: robots,

            food_spawn_accum: 0.0,
        }
    }

    pub fn tiles(&self) -> &GameTiles {
        &self.tiles
    }

    pub fn items(&self) -> &Items {
        &self.items
    }

    pub fn robots(&self) -> &Robots {
        &self.robots
    }

    pub fn update(&mut self, delta: f32, rng: &mut StdRng) {
        // Update all items
        self.items.update(&self.tiles, delta);

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
