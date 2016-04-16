extern crate tiled;
extern crate tungsten;
extern crate tungsten_glium2d;

mod model;
mod view;

use std::fs::File;
use std::path::Path;
use tungsten::{Framework, EventDispatcher, UpdateEvent};
use tungsten_glium2d::{Frontend2D, CloseRequestEvent, KeyboardInputEvent, Key, KeyState};
use model::GameModel;
use view::View;

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

fn main() {
    let mut file = File::open(&Path::new("assets/map.tmx")).unwrap();
    let map = tiled::parse(&mut file).unwrap();
    let model = GameModel::new(map);

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
