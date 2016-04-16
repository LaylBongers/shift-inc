extern crate tungsten;
extern crate tungsten_glium2d;

use tungsten::{Framework, EventDispatcher, UpdateEvent};
use tungsten_glium2d::{Frontend2D, CloseRequestEvent, FrameRenderInfo, KeyboardInputEvent, Key, KeyState, RenderTarget, Rectangle, View2D, TextureId};

struct GameModel {
    should_close: bool,
}

impl GameModel {
    fn new() -> Self {
        GameModel {
            should_close: false,
        }
    }

    fn update(&mut self, _delta: f32) {
    }

    fn close(&mut self) {
        self.should_close = true;
    }

    fn keep_running(&self) -> bool {
        !self.should_close
    }
}

fn close_request_handler(model: &mut GameModel, _event: &CloseRequestEvent) {
    model.close();
}

fn update_handler(model: &mut GameModel, event: &UpdateEvent) {
    model.update(event.delta);
}

fn keyboard_handler(model: &mut GameModel, event: &KeyboardInputEvent) {
    if event.state == KeyState::Pressed {
        match event.key {
            Key::Escape => model.close(),
            _ => ()
        }
    }
}

struct View {
}

impl View {
    fn new(_frontend: &mut Frontend2D<GameModel>) -> Self {
        // Load in textures

        View {
        }
    }

    fn render_world(&self, _model: &GameModel, info: &mut FrameRenderInfo) {
        let camera = info.game_camera([0.0, 0.0]);
        let _batch = camera.batch();
    }

    fn render_ui(&self, _model: &GameModel, info: &mut FrameRenderInfo) {
        let camera = info.game_camera([0.0, 0.0]);
        let _batch = camera.batch();
    }
}

impl View2D<GameModel> for View {
    fn render(&mut self, model: &GameModel, info: &mut FrameRenderInfo) {
        self.render_world(model, info);
        self.render_ui(model, info);
    }
}

fn main() {
    let model = GameModel::new();

    let mut event_dispatcher = EventDispatcher::new();
    event_dispatcher.add_handler(close_request_handler);
    event_dispatcher.add_handler(update_handler);
    event_dispatcher.add_handler(keyboard_handler);

    let mut frontend = Frontend2D::new();
    let view = View::new(&mut frontend);
    frontend.set_view(view);

    let framework = Framework::new(model, frontend, event_dispatcher);
    framework.run(|model| model.keep_running());
}
