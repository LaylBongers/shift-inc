use tungsten_glium2d::{Frontend2D, FrameRenderInfo, RenderTarget, View2D, TextureId, Rectangle};
use model::GameModel;

pub struct View {
    tiles: Vec<TextureId>,
    food: TextureId,
}

impl View {
    pub fn new(frontend: &mut Frontend2D<GameModel>) -> Self {
        // Load in textures
        let tiles = vec!(
            frontend.load_texture("./assets/background.png"),
            frontend.load_texture("./assets/foreground.png"),
        );

        View {
            tiles: tiles,
            food: frontend.load_texture("./assets/food.png"),
        }
    }

    fn render_world(&self, model: &GameModel, info: &mut FrameRenderInfo) {
        let camera = info.game_camera([1500.0, 5300.0]); // TODO: Move up!!!
        let batch = camera.batch();

        // Render the tiles
        let tiles = model.map().tiles();
        for x in 0..tiles.width() {
            for y in 0..tiles.height() {
                let tile = tiles.get(x, y).unwrap();
                let rect = Rectangle {
                    texture: self.tiles[tile.id as usize],
                    position: [128.0 * x as f32 + 64.0, 128.0 * y as f32 + 64.0],
                    size: [128.0, 128.0],
                };
                batch.rectangle(rect);
            }
        }

        // Render the items
        model.map().items().for_each(|item| {
            let rect = Rectangle {
                texture: self.food,
                position: [item.position[0] * 128.0, item.position[1] * 128.0],
                size: [32.0, 32.0],
            };
            batch.rectangle(rect);
        });
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
