use crate::ext::AnyNumber;
use rutie::{AnyObject, Class, Integer, NilClass, Object, RString, VerifiedObject};

wrappable_struct!(WaitInner, WaitWrapper, WAIT_WRAPPER);

class!(Wait);

class!(Waiter);

#[derive(Clone, Copy, Debug)]
pub enum WaitInner {
    Seconds(f32),
    Frames(usize),
    Done,
}

#[rustfmt::skip]
methods!(
    Wait,
    _itself,

    fn wait_to_string() -> RString {
        RString::new_utf8(&format!("{:?}", Into::<WaitInner>::into(_itself)))
    }
);

impl From<WaitInner> for Wait {
    fn from(inner: WaitInner) -> Self {
        Class::from_existing("Wait").wrap_data(inner, &*WAIT_WRAPPER)
    }
}

impl Into<WaitInner> for Wait {
    fn into(self) -> WaitInner { *self.get_data(&*WAIT_WRAPPER) }
}

impl VerifiedObject for Wait {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::from_existing("Wait")
    }

    fn error_message() -> &'static str { "Object is not of expected type Wait" }
}

#[rustfmt::skip]
methods!(
    Waiter,
    _itself,

    fn waiter_initialize(yielder: AnyObject) -> NilClass {
        _itself.instance_variable_set("@yielder", yielder.unwrap());
        NilClass::new()
    }

    fn wait_for_seconds(seconds: AnyNumber) -> NilClass {
        unsafe {
            _itself.instance_variable_get("@yielder").send("yield", &[
                Class::from_existing("Wait").wrap_data(
                    WaitInner::Seconds(seconds.unwrap().to_f32()),
                    &*WAIT_WRAPPER,
                ),
            ]);
        }
        NilClass::new()
    }

    fn wait_for_frames(frames: Integer) -> NilClass {
        unsafe {
            _itself.instance_variable_get("@yielder").send("yield", &[
                Class::from_existing("Wait").wrap_data(
                    WaitInner::Frames(frames.unwrap().to_i32() as usize),
                    &*WAIT_WRAPPER,
                ),
            ]);
        }
        NilClass::new()
    }

    fn wait_next_frame() -> NilClass {
        unsafe {
            _itself.instance_variable_get("@yielder").send("yield", &[
                Class::from_existing("Wait").wrap_data(
                    WaitInner::Frames(1),
                    &*WAIT_WRAPPER,
                ),
            ]);
        }
        NilClass::new()
    }

    fn wait_done() -> NilClass {
        unsafe {
            _itself.instance_variable_get("@yielder").send("yield", &[
                Class::from_existing("Wait").wrap_data(
                    WaitInner::Done,
                    &*WAIT_WRAPPER,
                ),
            ]);
        }
        NilClass::new()
    }
);

fn add_wait_class() {
    let mut class = Class::new("Wait", None);
    class.def("to_s", wait_to_string);
}

fn add_waiter_class() {
    let mut class = Class::new("Waiter", None);
    class.def("initialize", waiter_initialize);

    class.def("for_seconds", wait_for_seconds);
    class.def("for_frames", wait_for_frames);
    class.def("next_frame", wait_next_frame);
    class.def("done", wait_done);
}

pub fn add_classes() {
    add_wait_class();
    add_waiter_class();
}
