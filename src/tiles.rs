use tiled::Map;

#[derive(Debug)]
pub struct Tile {
    pub id: u32
}

impl Tile {
    fn from_raw_id(raw_id: u32) -> Self {
        let id = if raw_id == 0 {
            1 // Default tile, solid flesh
        } else {
            raw_id - 1 // Actual tile
        };

        Tile {
            id: id,
        }
    }
}

pub struct Tiles {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Tiles {
    pub fn load(map: &Map) -> Self {
        // Process the tiles
        let tiles_layer = map.layers.iter().find(|v| v.name == "Tiles").unwrap();
        let mut tiles = Vec::new();
        for row in &tiles_layer.tiles {
            for tile in row {
                tiles.push(Tile::from_raw_id(*tile));
            }
        }

        Tiles {
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

    pub fn for_each<F: FnMut(u32, u32, &Tile)>(&self, mut f: F) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let tile = self.get(x, y).unwrap();
                f(x, y, tile);
            }
        }
    }
}
