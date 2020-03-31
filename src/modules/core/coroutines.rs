use super::wait::WaitInner;
use rutie::{AnyObject, Class, Object, VerifiedObject, GC};
use std::time::Duration;

wrappable_struct!(
    CoroutinesInner,
    CoroutinesWrapper,
    COROUTINES_WRAPPER,
    mark(data) {
        data.coroutines.iter().map(|c| &c.block).for_each(GC::mark);
    }
);

pub struct Coroutine {
    pub wait: WaitInner,
    pub error: bool,
    pub block: AnyObject,
}

class!(Coroutines);

class!(CoroutineEnum);

pub struct CoroutinesInner {
    pending: Vec<Coroutine>,
    pub coroutines: Vec<Coroutine>,
}

impl Coroutine {
    pub fn schedule(&mut self, duration: Duration) {
        match self.wait {
            WaitInner::Seconds(ref mut seconds) => {
                *seconds -= duration.as_secs_f32();
            }
            WaitInner::Frames(ref mut frames) => {
                *frames = frames.saturating_sub(1);
            }
            WaitInner::Done => {}
        }
    }

    pub fn ready(&self) -> bool {
        match self.wait {
            WaitInner::Seconds(seconds) => seconds <= 0.0,
            WaitInner::Frames(frames) => frames == 0,
            WaitInner::Done => false,
        }
    }

    pub fn done(&self) -> bool {
        match self.wait {
            WaitInner::Seconds(_) => false,
            WaitInner::Frames(_) => false,
            WaitInner::Done => true,
        }
    }
}

impl Coroutines {
    pub fn new() -> Self {
        From::from(CoroutinesInner {
            pending: Vec::new(),
            coroutines: Vec::new(),
        })
    }

    pub fn push(&mut self, coroutine: Coroutine) {
        self.get_data_mut(&*COROUTINES_WRAPPER)
            .pending
            .push(coroutine);
    }

    pub fn tidy(&mut self) {
        let inner = self.get_data_mut(&*COROUTINES_WRAPPER);
        inner.coroutines.append(&mut inner.pending);
    }
}

impl AsMut<Vec<Coroutine>> for Coroutines {
    fn as_mut(&mut self) -> &mut Vec<Coroutine> {
        &mut self.get_data_mut(&*COROUTINES_WRAPPER).coroutines
    }
}

impl VerifiedObject for Coroutines {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::from_existing("Coroutines")
    }

    fn error_message() -> &'static str { "Object is not of type Coroutines" }
}

impl From<CoroutinesInner> for Coroutines {
    fn from(inner: CoroutinesInner) -> Self {
        Class::from_existing("Coroutines").wrap_data(inner, &*COROUTINES_WRAPPER)
    }
}

pub fn add_coroutines_class() { Class::new("Coroutines", None); }
