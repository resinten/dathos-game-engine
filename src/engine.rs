use crate::modules::core::CoreModule;
use crate::modules::draw::{BuildError as DrawBuildError, DrawModule};
use crate::modules::{EngineModule, GameState};
use rutie::{AnyException, Class, NilClass, Object, VM};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    DrawBuild(DrawBuildError),
    InvalidEntryScript(PathBuf),
    Update(UpdateError),
}

#[derive(Debug)]
pub enum UpdateError {
    Interrupt,
    Unknown(AnyException),
}

pub struct Engine<G>
where
    G: GameState,
{
    entry_script: PathBuf,
    game_state: G,
    modules: Vec<Box<dyn EngineModule<G>>>,
}

impl From<DrawBuildError> for Error {
    fn from(e: DrawBuildError) -> Self { Error::DrawBuild(e) }
}

impl From<UpdateError> for Error {
    fn from(e: UpdateError) -> Self { Error::Update(e) }
}

impl<G> Engine<G>
where
    G: GameState,
{
    pub fn build(entry_script: PathBuf, game_state: G) -> Self {
        Engine {
            entry_script,
            game_state,
            modules: Vec::new(),
        }
    }

    pub fn with_module<M>(mut self, module: M) -> Self
    where
        M: 'static + EngineModule<G>,
    {
        self.modules.push(box module);
        self
    }

    pub fn run(mut self) -> Result<(), Error> {
        VM::init();
        VM::init_loadpath();

        let draw_module = DrawModule::build(&self.game_state)?;
        self = self.with_module(CoreModule).with_module(draw_module);

        self.initialize()?;
        'game: loop {
            let result = self.update();
            match result {
                Ok(()) => {}
                Err(UpdateError::Interrupt) => {
                    println!("Interrupt requested");
                    break 'game;
                }
                Err(UpdateError::Unknown(e)) => {
                    println!("Unknown error: {:?}", e);
                    break 'game;
                }
            }
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), Error> {
        let modules = &mut self.modules;
        let game_state = &mut self.game_state;
        modules.iter_mut().for_each(|m| m.init(game_state));
        let result = VM::protect(|| {
            VM::require(
                &self
                    .entry_script
                    .to_str()
                    .ok_or(Error::InvalidEntryScript(self.entry_script.clone()))
                    .unwrap(),
            );
            NilClass::new().to_any_object()
        });
        if let Err(_) = result {
            println!("Error: {:?}", VM::error_info());
        }
        Ok(())
    }

    fn update(&mut self) -> Result<(), UpdateError> {
        let modules = &mut self.modules;
        let game_state = &mut self.game_state;
        let result = VM::protect(|| {
            modules.iter_mut().for_each(|m| m.pre_update(game_state));
            modules.iter_mut().for_each(|m| m.update(game_state));
            modules.iter_mut().for_each(|m| m.post_update(game_state));
            NilClass::new().to_any_object()
        });
        match result {
            Ok(_) => Ok(()),
            Err(_) => match VM::error_info() {
                Ok(e) => Err(
                    if e.class()
                        .ancestors()
                        .contains(&Class::from_existing("Interrupt"))
                    {
                        UpdateError::Interrupt
                    } else {
                        UpdateError::Unknown(e)
                    },
                ),
                Err(_) => Ok(()),
            },
        }
    }
}
