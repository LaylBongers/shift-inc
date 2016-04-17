extern crate tiled;
extern crate rand;
#[macro_use] extern crate enum_primitive;
extern crate cgmath;
extern crate tungsten;
extern crate tungsten_glium2d;

mod items;
mod map;
mod model;
mod robots;
mod tiles;
mod view;

use std::fs::File;
use std::path::Path;
use cgmath::Vector2;
use tungsten::{Framework, EventDispatcher, UpdateEvent};
use tungsten_glium2d::{Frontend2D, CloseRequestEvent, KeyboardInputEvent, Key, ElementState, MouseMoveEvent, MouseButton, MouseButtonEvent};
use model::{GameModel, GameButton};
use view::View;

fn close_request_handler(model: &mut GameModel, _event: &CloseRequestEvent) {
    model.close();
}

fn update_handler(model: &mut GameModel, event: &UpdateEvent) {
    model.update(event.delta);
}

fn keyboard_handler(model: &mut GameModel, event: &KeyboardInputEvent) {
    let pressed = event.state == ElementState::Pressed;

    // Check for the escape key
    if pressed {
        match event.key {
            Key::Escape => model.close(),
            _ => ()
        }
    }

    // Relay all key changes
    match event.key {
        Key::W => model.handle_keychange(GameButton::CameraUp, pressed),
        Key::A => model.handle_keychange(GameButton::CameraLeft, pressed),
        Key::S => model.handle_keychange(GameButton::CameraDown, pressed),
        Key::D => model.handle_keychange(GameButton::CameraRight, pressed),
        _ => ()
    }
}

fn mouse_move_handler(model: &mut GameModel, event: &MouseMoveEvent) {
    model.handle_mouse_move(Vector2::from(event.position).cast());
}

fn mouse_button_handler(model: &mut GameModel, event: &MouseButtonEvent) {
    let pressed = event.state == ElementState::Pressed;

    match event.button {
        MouseButton::Left => model.handle_keychange(GameButton::Interact, pressed),
        _ => ()
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
    event_dispatcher.add_handler(mouse_move_handler);
    event_dispatcher.add_handler(mouse_button_handler);

    let mut frontend = Frontend2D::new();
    let view = View::new(&mut frontend);
    frontend.set_view(view);

    let framework = Framework::new(model, frontend, event_dispatcher);
    framework.run(|model| model.keep_running());
}
