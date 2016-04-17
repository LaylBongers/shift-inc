use tiled::Map;
use rand::StdRng;
use cgmath::{Vector2, EuclideanVector};
use map::GameMap;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    pub enum GameButton {
        CameraUp,
        CameraLeft,
        CameraDown,
        CameraRight,
        Interact,
    }
}

pub struct InputState {
    keys: Vec<bool>,
    hover_tile: Vector2<u32>,
}

impl InputState {
    fn new() -> Self {
        InputState {
            keys: vec![false; GameButton::Interact as usize + 1],
            hover_tile: Vector2::new(0, 0),
        }
    }

    fn set(&mut self, key: GameButton, state: bool) {
        self.keys[key as usize] = state;
    }

    fn get(&self, key: GameButton) -> bool {
        self.keys[key as usize]
    }

    fn get_axis(&self, left: GameButton, right: GameButton) -> f32 {
        let mut value = 0.0;
        if self.get(left) { value -= 1.0; }
        if self.get(right) { value += 1.0; }
        value
    }

    fn get_axes_normalized(&self,
        left: GameButton, right: GameButton,
        down: GameButton, up: GameButton)
    -> Vector2<f32> {
        let mut value = Vector2::new(0.0, 0.0);
        value.x += self.get_axis(left, right);
        value.y += self.get_axis(down, up);

        if value.magnitude2() != 0.0 {
            value.normalize()
        } else {
            value
        }
    }

    fn process_mouse(&mut self, screen_pos: Vector2<u32>, camera: &GameCamera) {
        let relative_to_center = screen_pos.cast::<f32>() - Vector2::new(1280.0/2.0, 720.0/2.0);
        let mut relative_to_center_world = relative_to_center / 128.0;
        relative_to_center_world.y = -relative_to_center_world.y; // Have to flip this axis
        let world = camera.position + relative_to_center_world;

        self.hover_tile = world.cast();
    }

    pub fn get_hover_tile(&self) -> Vector2<u32> {
        self.hover_tile
    }
}

pub struct GameCamera {
    position: Vector2<f32>,
}

impl GameCamera {
    fn new() -> Self {
        GameCamera {
            position: Vector2::new(25.0, 32.0)
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn update(&mut self, delta: f32, input: &InputState) {
        let axes = input.get_axes_normalized(
            GameButton::CameraLeft, GameButton::CameraRight, GameButton::CameraDown, GameButton::CameraUp
        );
        self.position = self.position + (axes * delta * 2.0);
    }
}

struct BuildingBehavior {
    prev_button: bool
}

impl BuildingBehavior {
    fn new() -> Self {
        BuildingBehavior {
            prev_button: false
        }
    }

    fn update(&mut self, input: &InputState, map: &mut GameMap) {
        let current_button = input.get(GameButton::Interact);

        if current_button != self.prev_button && !current_button {
            let tile_pos = input.get_hover_tile();

            // If the tile's already a 2 or a 3, don't do anything
            let current_class = map.get_tile(tile_pos).unwrap();
            if current_class != 2 && current_class != 3 {
                // Create a construction where the mouse is
                map.start_construction(tile_pos, 2);
            }
        }

        self.prev_button = current_button;
    }
}

pub struct GameModel {
    should_close: bool,
    map: GameMap,
    camera: GameCamera,

    input: InputState,
    building: BuildingBehavior,
    rng: StdRng,
}

impl GameModel {
    pub fn new(map: Map) -> Self {
        let mut rng = StdRng::new().unwrap();
        let game_map = GameMap::load(map, &mut rng);

        GameModel {
            should_close: false,
            map: game_map,
            camera: GameCamera::new(),

            input: InputState::new(),
            building: BuildingBehavior::new(),
            rng: rng,
        }
    }

    pub fn keep_running(&self) -> bool {
        !self.should_close
    }

    pub fn map(&self) -> &GameMap {
        &self.map
    }

    pub fn camera(&self) -> &GameCamera {
        &self.camera
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }

    pub fn update(&mut self, delta: f32) {
        self.building.update(&self.input, &mut self.map);
        self.camera.update(delta, &self.input);
        self.map.update(delta, &mut self.rng);
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn handle_keychange(&mut self, key: GameButton, state: bool) {
        self.input.set(key, state);
    }

    pub fn handle_mouse_move(&mut self, position: Vector2<u32>) {
        self.input.process_mouse(position, &self.camera);
    }
}
