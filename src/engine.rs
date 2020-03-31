use crate::modules::core::CoreModule;
use crate::modules::draw::{BuildError as DrawBuildError, DrawModule, WindowOptions};
use crate::modules::EngineModule;
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

pub struct Engine {
    entry_script: PathBuf,
    window_options: WindowOptions,
    modules: Vec<Box<dyn EngineModule>>,
}

pub struct EngineOptions {
    pub window: WindowOptions,
    pub entry_script: PathBuf,
}

impl From<DrawBuildError> for Error {
    fn from(e: DrawBuildError) -> Self { Error::DrawBuild(e) }
}

impl From<UpdateError> for Error {
    fn from(e: UpdateError) -> Self { Error::Update(e) }
}

impl Engine {
    pub fn build(options: EngineOptions) -> Self {
        Engine {
            entry_script: options.entry_script,
            window_options: options.window,
            modules: Vec::new(),
        }
    }

    pub fn with_module<M>(mut self, module: M) -> Self
    where
        M: 'static + EngineModule,
    {
        self.modules.push(box module);
        self
    }

    pub fn run(mut self) -> Result<(), Error> {
        VM::init();
        VM::init_loadpath();

        let window_options = self.window_options.clone();
        self = self
            .with_module(CoreModule)
            .with_module(DrawModule::build(window_options)?);

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
        self.modules.iter_mut().for_each(|m| m.init());
        VM::require(
            &self
                .entry_script
                .to_str()
                .ok_or(Error::InvalidEntryScript(self.entry_script.clone()))?,
        );
        Ok(())
    }

    fn update(&mut self) -> Result<(), UpdateError> {
        let result = VM::protect(|| {
            self.modules.iter_mut().for_each(|m| m.pre_update());
            self.modules.iter_mut().for_each(|m| m.update());
            self.modules.iter_mut().for_each(|m| m.post_update());
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
