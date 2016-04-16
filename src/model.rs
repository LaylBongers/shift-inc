use tiled::Map;
use rand::StdRng;
use cgmath::{Vector2, EuclideanVector};
use map::GameMap;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    pub enum GameKey {
        CameraUp,
        CameraLeft,
        CameraDown,
        CameraRight,
    }
}

struct InputState {
    keys: Vec<bool>
}

impl InputState {
    fn new() -> Self {
        InputState {
            keys: vec![false; GameKey::CameraRight as usize + 1]
        }
    }

    fn set(&mut self, key: GameKey, state: bool) {
        self.keys[key as usize] = state;
    }

    fn get(&self, key: GameKey) -> bool {
        self.keys[key as usize]
    }

    fn get_axis(&self, left: GameKey, right: GameKey) -> f32 {
        let mut value = 0.0;
        if self.get(left) { value -= 1.0; }
        if self.get(right) { value += 1.0; }
        value
    }

    fn get_axes_normalized(&self,
        left: GameKey, right: GameKey,
        down: GameKey, up: GameKey)
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
            GameKey::CameraLeft, GameKey::CameraRight, GameKey::CameraDown, GameKey::CameraUp
        );
        self.position = self.position + (axes * delta * 2.0);
    }
}

pub struct GameModel {
    should_close: bool,
    map: GameMap,
    camera: GameCamera,

    input: InputState,
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

    pub fn update(&mut self, delta: f32) {
        self.camera.update(delta, &self.input);
        self.map.update(delta, &mut self.rng);
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn handle_keychange(&mut self, key: GameKey, state: bool) {
        self.input.set(key, state);
    }
}
