pub mod core;
pub mod draw;

pub trait EngineModule {
    fn init(&mut self) {}
    fn pre_update(&mut self) {}
    fn update(&mut self) {}
    fn post_update(&mut self) {}
}
