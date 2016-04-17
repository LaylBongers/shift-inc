use cgmath::Vector2;
use tiled::{Map, Object};
use rand::{StdRng, Rng};
use items::{Item, Items, ItemState};
use robots::{Robots, Robot, WorkQueue, WorkEntry};
use tiles::Tiles;

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
            position: Vector2::new(x, y),
            lifetime: 60.0,
            state: ItemState::Falling,
            claimed: false,
        }
    }
}

pub struct GameMap {
    tiles: Tiles,
    food_spawners: Vec<FoodSpawner>,

    items: Items,
    robots: Robots,

    food_spawn_accum: f32,
    work_queue: WorkQueue,
}

impl GameMap {
    pub fn load(map: Map, rng: &mut StdRng) -> Self {
        // Initialize the work queue
        let mut work = WorkQueue::new();

        // Should have 1 layer and 1 object group
        assert_eq!(map.layers.len(), 1);
        assert_eq!(map.object_groups.len(), 1);

        // Load in the tiles
        let tiles = Tiles::load(&map, &mut work);

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

        // Spawn a single robot for the first under construction tile we find
        // TODO: Temporarily, it's two for the sake of testing
        let mut robots = Robots::new();
        'spawn_robot: for x in 0..tiles.width() {
            for y in 0..tiles.height() {
                let tile = tiles.get(x, y).unwrap();
                if !tile.is_under_construction() { continue; }

                robots.add(Robot::new(Vector2::new(x as f32 + 0.5, y as f32 + 0.5)));
                if robots.amount() >= 2 {
                    break 'spawn_robot;
                }
            }
        }

        // Create the actual struct
        let mut map = GameMap {
            tiles: tiles,
            food_spawners: food_spawners,

            items: Items::new(),
            robots: robots,

            food_spawn_accum: 0.0,
            work_queue: work,
        };

        // Spawn some food and advance time before the first frame
        for _ in 0..6 {
            for _ in 0..4 {
                map.items.update(&map.tiles, 0.2);
                map.items.update(&map.tiles, 0.2);
                map.items.update(&map.tiles, 0.2);
                map.items.update(&map.tiles, 0.2);
                map.items.update(&map.tiles, 0.2);
            }

            map.spawn_food(rng);
        }

        map
    }

    pub fn tiles(&self) -> &Tiles {
        &self.tiles
    }

    pub fn items(&self) -> &Items {
        &self.items
    }

    pub fn robots(&self) -> &Robots {
        &self.robots
    }

    pub fn start_construction(&mut self, pos: Vector2<u32>, class: u32) {
        let tile = self.tiles.get_mut(pos.x, pos.y).unwrap();

        // Can't overwrite an existing construction
        if tile.is_under_construction() {
            return;
        }

        // Set the tile to under construction
        tile.set_construction(class);

        // Create a work item for that tile
        self.work_queue.publish(WorkEntry::new(pos));
    }

    pub fn get_tile(&self, pos: Vector2<u32>) -> Option<u32> {
        self.tiles.get(pos.x, pos.y).map(|v| v.class())
    }

    pub fn update(&mut self, delta: f32, rng: &mut StdRng) {
        // Update all items
        self.items.update(&self.tiles, delta);

        // Spawn a food blob if enough time has passed
        self.food_spawn_accum += delta;
        while self.food_spawn_accum > 4.0 {
            self.food_spawn_accum -= 4.0;
            self.spawn_food(rng);
        }

        // Update all the robots
        self.robots.update(delta, &mut self.items, &mut self.tiles, &mut self.work_queue, rng);
    }

    fn spawn_food(&mut self, rng: &mut StdRng) {
        assert_eq!(self.food_spawners.len(), 1);
        let spawner = &self.food_spawners[0];
        let item = spawner.spawn(rng);
        self.items.add(item);
    }
}
