use super::game_object::GameObject;
use super::vector::VectorData;
use crate::ext::AnyNumber;
use nalgebra::Vector2;
use quanta::{Clock, Instant};
use rutie::{AnyObject, Boolean, Class, Float, Integer, Module, Object, GC};
use std::time::Duration;

wrappable_struct!(
    GameInner,
    GameWrapper,
    GAME_WRAPPER,
    mark(data) {
        data.pending_creates.iter().for_each(GC::mark);
        data.pending_deletes.iter().for_each(GC::mark);
        data.game_objects.iter().for_each(GC::mark);
    }
);

module!(Game);

pub struct GameInner {
    pub camera: Vector2<f32>,
    pub pending_creates: Vec<GameObject>,
    pub pending_deletes: Vec<GameObject>,
    pub game_objects: Vec<GameObject>,
    pub time: Time,
}

#[derive(Clone, Debug)]
pub struct Time {
    pub clock: Clock,
    pub frame: u64,
    pub delta: Duration,
    pub now: Instant,
    pub start: Instant,
}

#[rustfmt::skip]
methods!(
    Game,
    _itself,

    fn get_camera() -> VectorData {
        _itself.instance_variable_get("@inner").get_data(&*GAME_WRAPPER).camera.into()
    }

    fn set_camera(camera: VectorData) -> VectorData {
        let mut inner = _itself.instance_variable_get("@inner");
        let game_inner = inner.get_data_mut(&*GAME_WRAPPER);
        game_inner.camera = camera.unwrap().into();
        game_inner.camera.into()
    }

    fn create_object(object: GameObject) -> GameObject {
        let object = object.unwrap();
        _itself
            .instance_variable_get("@inner")
            .get_data_mut(&*GAME_WRAPPER).pending_creates.push(object.clone());
        object
    }

    fn delete_object(object: GameObject) -> Boolean {
        Boolean::new(if let Ok(object) = object {
            _itself
                .instance_variable_get("@inner")
                .get_data_mut(&*GAME_WRAPPER).pending_deletes.push(object);
            true
        } else {
            false
        })
    }

    fn get_frame() -> Integer {
        Integer::new(
            _itself
                .instance_variable_get("@inner")
                .get_data(&*GAME_WRAPPER)
                .time
                .frame as i64
        )
    }

    fn get_delta_time() -> Float {
        Float::new(
            _itself
                .instance_variable_get("@inner")
                .get_data(&*GAME_WRAPPER)
                .time
                .delta
                .as_secs_f64()
        )
    }

    fn get_time() -> Float {
        let inner = _itself.instance_variable_get("@inner");
        let game_data = inner.get_data(&*GAME_WRAPPER);
        Float::new(
            game_data
                .time
                .now
                .saturating_duration_since(game_data.time.start)
                .as_secs_f64()
        )
    }

    fn time_since(t: AnyNumber) -> Float {
        let inner = _itself.instance_variable_get("@inner");
        let game_data = inner.get_data(&*GAME_WRAPPER);
        let now = game_data
            .time
            .now
            .saturating_duration_since(game_data.time.start)
            .as_secs_f32();
        Float::new((now - t.unwrap().to_f32()) as f64)
    }
);

impl GameInner {
    fn new() -> Self {
        let clock = Clock::new();
        let last_instant = clock.now();
        GameInner {
            camera: Vector2::new(0.0, 0.0),
            pending_creates: Vec::new(),
            pending_deletes: Vec::new(),
            game_objects: Vec::new(),
            time: Time {
                clock,
                frame: 0,
                delta: Duration::from_millis(0),
                now: last_instant,
                start: last_instant,
            },
        }
    }
}

pub fn add_game_module() {
    let mut module = Module::new("Game");

    let inner: AnyObject =
        Class::from_existing("Object").wrap_data(GameInner::new(), &*GAME_WRAPPER);
    module.instance_variable_set("@inner", inner);

    module.def_self("camera", get_camera);
    module.def_self("frame", get_frame);
    module.def_self("delta_time", get_delta_time);
    module.def_self("time", get_time);
    module.def_self("time_since", time_since);

    module.def_self("camera=", set_camera);

    // module.def_self("axis_direction", axis_direction);
    // module.def_self("key_down", key_down);
    // module.def_self("key_hit", key_hit);

    module.def_self("create!", create_object);
    module.def_self("delete!", delete_object);
}
