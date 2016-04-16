use cgmath::Vector2;
use tiled::Map;
use robots::{WorkQueue, WorkEntry};

#[derive(Debug)]
pub struct Tile {
    class: u32,
    construction: Option<(u32, f32)>, // class, time remaining
}

impl Tile {
    fn from_raw_id(raw_class: u32) -> Self {
        // Get the actual ID
        let mut class = {
            if raw_class == 0 {
                1 // If there's no tile here, use the default tile, solid flesh
            } else {
                raw_class - 1 // Actual tile
            }
        };

        // See if we need to flip this tile to under construction
        // We do this for the initial structues and let the robots build it
        let mut construction = None;
        if class == 2 || class == 3 {
            construction = Some((class, 2.0));
            class = 0; // Empty
        }

        Tile {
            class: class,
            construction: construction,
        }
    }

    pub fn class(&self) -> u32 {
        self.class
    }

    pub fn set_class(&mut self, class: u32) {
        self.construction = None;
        self.class = class;
    }

    pub fn is_under_construction(&self) -> bool {
        if let Some(_) = self.construction {
            true
        } else {
            false
        }
    }

    /*pub fn under_construction_class(&self) -> Option<u32> {
        self.construction.map(|v| v.0)
    }*/

    /*pub fn set_construction(&mut self, class: u32) {
        self.construction = Some(class);
    }*/

    pub fn apply_build_time(&mut self, delta: f32) -> bool {
        // Perform the building
        let (done, new_class) = {
            let constr: &mut (u32, f32) = &mut self.construction.as_mut().unwrap();
            constr.1 = constr.1 - delta;

            // If we reached 0 in construction time, we're done
            (constr.1 <= 0.0, constr.0)
        };

        // If we're done, switch to the new tile
        if done {
            self.set_class(new_class);
            true
        } else {
            false
        }
    }
}

pub struct Tiles {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Tiles {
    pub fn load(map: &Map, work: &mut WorkQueue) -> Self {
        // Process the tiles
        let tiles_layer = map.layers.iter().find(|v| v.name == "Tiles").unwrap();
        let mut tiles = Vec::new();
        for row in &tiles_layer.tiles {
            for tile in row {
                tiles.push(Tile::from_raw_id(*tile));
            }
        }

        let tiles = Tiles {
            width: map.width,
            height: map.height,
            tiles: tiles,
        };

        // Spawn work items for each under construction tile
        tiles.for_each(|x, y, tile| {
            if tile.is_under_construction() {
                work.publish(WorkEntry::new(Vector2::new(x, y)));
            }
        });

        tiles
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

    pub fn get(&self, x: u32, y: u32) -> Option<&Tile> {
        if y >= self.height || x >= self.width {
            None
        } else {
            // Grain is 1,0 -> 2,0 -> 0,1 x increases first
            let tile = self.tiles.get(self.position_for(x, y));
            assert!(tile.is_some(), "Tile at {},{} should have been some!", x, y);
            tile
        }
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if y >= self.height || x >= self.width {
            None
        } else {
            // Grain is 1,0 -> 2,0 -> 0,1 x increases first
            let pos = self.position_for(x, y);
            let tile = self.tiles.get_mut(pos);
            assert!(tile.is_some(), "Tile at {},{} should have been some!", x, y);
            tile
        }
    }

    pub fn for_each<F: FnMut(u32, u32, &Tile)>(&self, mut f: F) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let tile = self.get(x, y).unwrap();
                f(x, y, tile);
            }
        }
    }
}
