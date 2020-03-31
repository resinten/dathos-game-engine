use super::coroutines::{Coroutine, Coroutines};
use super::game::GAME_WRAPPER;
use super::wait::{Wait, WaitInner};
use crate::ext::AnyNumber;
use rutie::{Boolean, Class, Module, NilClass, Object, VerifiedObject, VM};

class!(GameObject);

impl Clone for GameObject {
    fn clone(&self) -> Self {
        GameObject {
            value: self.value.clone(),
        }
    }
}

impl GameObject {
    pub fn on_start(&self) {
        let result = self.protect_send("on_start!", &[]);
        if let Err(e) = result {
            println!("on_start: {:?}", e);
        }
    }

    pub fn update(&self) {
        let result = self.protect_send("update!", &[]);
        if let Err(e) = result {
            println!("update: {:?}", e);
        }
    }

    pub fn on_delete(&self) {
        let result = self.protect_send("on_delete!", &[]);
        if let Err(e) = result {
            println!("on_delete: {:?}", e);
        }
    }

    pub fn execute_coroutines(&self) {
        let game_inner = Module::from_existing("Game").instance_variable_get("@inner");
        let elapsed = game_inner.get_data(&*GAME_WRAPPER).time.delta;
        let mut coroutines_data = self
            .instance_variable_get("@coroutines")
            .try_convert_to::<Coroutines>()
            .unwrap();
        let coroutines = AsMut::<Vec<Coroutine>>::as_mut(&mut coroutines_data);
        coroutines.iter_mut().for_each(|c| c.schedule(elapsed));
        coroutines.iter_mut().filter(|c| c.ready()).for_each(|c| {
            let yielded = c.block.protect_send("next", &[]);
            match yielded {
                Ok(yielded) => {
                    let wait = yielded.try_convert_to::<Wait>().unwrap().into();
                    c.error = false;
                    c.wait = wait;
                }
                Err(e) => {
                    println!("execute_coroutines: {:?}", e);
                    c.error = true;
                    c.wait = WaitInner::Done;
                }
            }
        });
        coroutines.retain(|c| !c.error && !c.done());
    }

    pub fn tidy_coroutines(&self) {
        let mut coroutines_data = self
            .instance_variable_get("@coroutines")
            .try_convert_to::<Coroutines>()
            .unwrap();
        coroutines_data.tidy();
    }
}

#[rustfmt::skip]
methods!(
    GameObject,
    _itself,

    fn run() -> Boolean {
        Boolean::new(if VM::is_block_given() {
            let block = Module::from_existing("GameUtils")
                .protect_send("make_coroutine", &[VM::block_proc().to_any_object()])
                .unwrap();
            let mut coroutines = _itself
                .instance_variable_get("@coroutines")
                .try_convert_to::<Coroutines>()
                .unwrap_or_else(|_| Coroutines::new());
            coroutines.push(Coroutine {
                wait: WaitInner::Frames(0),
                error: false,
                block,
            });
            _itself.instance_variable_set("@coroutines", coroutines);
            true
        } else {
            false
        })
    }

    fn run_for(duration: AnyNumber) -> Boolean {
        Boolean::new(if VM::is_block_given() {
            let block = Module::from_existing("GameUtils")
                .protect_send(
                    "make_run_for",
                    &[
                        duration.unwrap().to_any_object(),
                        VM::block_proc().to_any_object()
                    ]
                )
                .unwrap();
            let mut coroutines = _itself
                .instance_variable_get("@coroutines")
                .try_convert_to::<Coroutines>()
                .unwrap_or_else(|_| Coroutines::new());
            coroutines.push(Coroutine {
                wait: WaitInner::Frames(0),
                error: false,
                block,
            });
            _itself.instance_variable_set("@coroutines", coroutines);
            true
        } else {
            false
        })
    }

    fn empty_method() -> NilClass {
        NilClass::new()
    }
);

impl VerifiedObject for GameObject {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object
            .class()
            .ancestors()
            .contains(&Class::from_existing("GameObject"))
    }

    fn error_message() -> &'static str { "Object is not type of class GameObject" }
}

pub fn add_game_object_class() {
    let mut class = Class::new("GameObject", None);

    class.attr_accessor("entity");
    class.attr_accessor("transform");
    class.attr_accessor("velocity");

    class.attr_accessor("collider");
    class.attr_accessor("collision_mask");

    class.def("run!", run);
    class.def("run_for!", run_for);

    class.def("on_start!", empty_method);
    class.def("update!", empty_method);
    class.def("on_delete!", empty_method);
    class.def("on_collision!", empty_method);
}
