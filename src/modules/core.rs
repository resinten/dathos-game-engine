pub use self::color::ColorData;
pub use self::coroutines::{Coroutine, Coroutines};
use self::game::GAME_WRAPPER;
pub use self::game_object::GameObject;
pub use self::input::{Input, INPUT_WRAPPER};
pub use self::vector::VectorData;
use super::EngineModule;
use rutie::{Module, Object, VM};

mod color;
mod coroutines;
mod game;
mod game_object;
mod input;
mod rotation;
mod transform;
mod vector;
mod wait;

const GAME_UTILS_MODULE: &str = include_str!("./core/game_utils.rb");

pub struct CoreModule;

impl CoreModule {
    fn handle_pending_deletes(&mut self) {
        let mut inner = Module::from_existing("Game").instance_variable_get("@inner");
        let game = inner.get_data_mut(&*GAME_WRAPPER);
        let mut deletes = Vec::new();
        deletes.append(&mut game.pending_deletes);
        game.game_objects.retain(|o| {
            let should_remove = deletes.contains(o);
            if should_remove {
                o.on_delete();
            }
            !should_remove
        });
    }

    fn handle_pending_creates(&mut self) {
        let mut inner = Module::from_existing("Game").instance_variable_get("@inner");
        let game = inner.get_data_mut(&*GAME_WRAPPER);
        game.pending_creates.iter_mut().for_each(|game_object| {
            let coroutines = game_object
                .instance_variable_get("@coroutines")
                .try_convert_to::<Coroutines>()
                .unwrap_or_else(|_| Coroutines::new());
            game_object.instance_variable_set("@coroutines", coroutines);
            game_object.on_start();
        });
        game.game_objects.append(&mut game.pending_creates);
    }
}

impl EngineModule for CoreModule {
    fn init(&mut self) {
        let _ = VM::eval(GAME_UTILS_MODULE);

        self::color::add_color_class();
        self::rotation::add_rotation_module();
        self::coroutines::add_coroutines_class();
        self::vector::add_vector_class();
        self::transform::add_transform_class();
        self::wait::add_classes();
        self::game_object::add_game_object_class();
        self::game::add_game_module();
        self::input::add_input_module();
    }

    fn pre_update(&mut self) {
        let mut inner = Module::from_existing("Game").instance_variable_get("@inner");
        let mut game = inner.get_data_mut(&*GAME_WRAPPER);
        let now = game.time.clock.now();
        game.time.frame += 1;
        game.time.delta = now - game.time.now;
        game.time.now = now;
    }

    fn update(&mut self) {
        let inner = Module::from_existing("Game").instance_variable_get("@inner");
        let game = inner.get_data(&*GAME_WRAPPER);
        game.game_objects.iter().for_each(GameObject::update);
        game.game_objects
            .iter()
            .for_each(GameObject::execute_coroutines);
        game.game_objects
            .iter()
            .for_each(GameObject::tidy_coroutines);
    }

    fn post_update(&mut self) {
        self.handle_pending_deletes();
        self.handle_pending_creates();
    }
}
