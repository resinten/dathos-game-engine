use self::primitive::{PrimitiveProgram, PrimitiveShaderInterface};
use self::sprite::{SpriteData, SpriteProgram, SpriteProgramBase, SpriteShaderInterface};
use self::text::{TextProgram, TextProgramBase};
use super::core::INPUT_WRAPPER;
use super::EngineModule;
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::shader::program::{Program, ProgramError};
use luminance::tess::{Mode, Tess, TessBuilder, TessError};
use luminance::texture::{DepthComparison, Dim2, MagFilter, MinFilter, Sampler, Wrap};
use luminance_glfw::{
    Action, GlfwSurface, GlfwSurfaceError, Key, Surface, WindowDim, WindowEvent, WindowOpt,
};
use nalgebra::{Vector2, Vector4};
use rutie::{Module, Object};

mod primitive;
mod ruby;
mod sprite;
mod text;

pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 360;

const GENERIC_VERTEX_SHADER: &str = include_str!("./draw/generic_vs.glsl");

const PRIMITIVE_FRAGMENT_SHADER: &str = include_str!("./draw/primitive_fs.glsl");
const SPRITE_FRAGMENT_SHADER: &str = include_str!("./draw/sprite_fs.glsl");

const SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::LinearMipmapLinear,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

#[derive(Debug)]
pub enum BuildError {
    Surface(GlfwSurfaceError),
    Program(ProgramError),
    Tess(TessError),
}

#[derive(Clone)]
pub struct WindowOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct ObjectGeometry {
    pub brighten: Vector4<f32>,
    pub darken: Vector4<f32>,
    pub depth: f32,
    pub desaturation: f32,
    pub origin: Vector2<f32>,
    pub position: Vector2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
}

pub struct DrawModule<'a> {
    surface: GlfwSurface,
    backbuffer: Framebuffer<Dim2, (), ()>,

    primitive_program: Program<(), (), PrimitiveShaderInterface>,
    sprite_base: SpriteProgramBase,
    sprite_program: Program<(), (), SpriteShaderInterface>,
    text_base: TextProgramBase<'a>,
    tess: Tess,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Arc {
        depth: f32,
        color: [f32; 4],
        radius: f32,
        thickness: f32,
        from: f32,
        to: f32,
        position: Vector2<f32>,
    },
    Circle {
        depth: f32,
        color: [f32; 4],
        radius: f32,
        position: Vector2<f32>,
    },
    Line {
        depth: f32,
        color: [f32; 4],
        thickness: f32,
        from: Vector2<f32>,
        to: Vector2<f32>,
    },
    Rectangle {
        depth: f32,
        color: [f32; 4],
        width: f32,
        height: f32,
        position: Vector2<f32>,
        rotation: f32,
    },
    Sprite,
    Text {
        depth: f32,
        color: [f32; 4],
        position: Vector2<f32>,
        size: f32,
        text: String,
    },
}

impl From<GlfwSurfaceError> for BuildError {
    fn from(e: GlfwSurfaceError) -> Self { BuildError::Surface(e) }
}

impl From<ProgramError> for BuildError {
    fn from(e: ProgramError) -> Self { BuildError::Program(e) }
}

impl From<TessError> for BuildError {
    fn from(e: TessError) -> Self { BuildError::Tess(e) }
}

impl Default for ObjectGeometry {
    fn default() -> Self {
        ObjectGeometry {
            brighten: [0.0, 0.0, 0.0, 0.0].into(),
            darken: [1.0, 1.0, 1.0, 1.0].into(),
            depth: 1.0,
            desaturation: 0.0,
            origin: [0.5, 0.5].into(),
            position: [0.0, 0.0].into(),
            rotation: 0.0,
            scale: [1.0, 1.0].into(),
        }
    }
}

impl<'a> DrawModule<'a> {
    pub fn build(options: WindowOptions) -> Result<Self, BuildError> {
        let mut surface = GlfwSurface::new(
            WindowDim::Windowed(options.width, options.height),
            &options.title,
            WindowOpt::default(),
        )?;
        let backbuffer = surface.back_buffer().unwrap();

        let text_base = TextProgramBase::new(&mut surface);
        let tess = TessBuilder::new(&mut surface)
            .set_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()?;

        Ok(DrawModule {
            surface,
            backbuffer,

            primitive_program: Program::<(), (), PrimitiveShaderInterface>::from_strings(
                None,
                GENERIC_VERTEX_SHADER,
                None,
                PRIMITIVE_FRAGMENT_SHADER,
            )?
            .ignore_warnings(),
            sprite_base: SpriteProgramBase::new(),
            sprite_program: Program::<(), (), SpriteShaderInterface>::from_strings(
                None,
                GENERIC_VERTEX_SHADER,
                None,
                SPRITE_FRAGMENT_SHADER,
            )?
            .ignore_warnings(),
            text_base,
            tess,
        })
    }

    fn clear_commands(&mut self) {
        let queue = Module::from_existing("Draw")
            .instance_variable_get("@queue")
            .try_convert_to::<self::ruby::DrawQueue>();
        if let Ok(mut queue) = queue {
            AsMut::<Vec<DrawCommand>>::as_mut(&mut queue).clear();
        }
    }

    fn handle_key_event(&self, key: Key, action: Action) {
        let mut input = Module::from_existing("Input").instance_variable_get("@input");
        let input_inner = input.get_data_mut(&*INPUT_WRAPPER);
        input_inner.handle_key_event(key, action);
    }

    fn prepare_render(&mut self) {
        let queue = Module::from_existing("Draw")
            .instance_variable_get("@queue")
            .try_convert_to::<self::ruby::DrawQueue>();
        if let Ok(mut queue) = queue {
            let commands = AsMut::<Vec<DrawCommand>>::as_mut(&mut queue);
            TextProgram {
                sprite: &self.sprite_base,
                text: &mut self.text_base,
                program: &self.sprite_program,
                tess: &self.tess,
            }
            .prepare_render(&mut self.surface, commands);
        }
    }

    fn render(&mut self) {
        let surface = &mut self.surface;
        let primitive_program = &self.primitive_program;
        let sprite_base = &self.sprite_base;
        let sprite_program = &self.sprite_program;
        let text_base = &mut self.text_base;
        let tess = &self.tess;

        let pipeline_state = PipelineState::new()
            .enable_clear_color(true)
            .set_clear_color([0.0, 0.0, 0.0, 0.0]);

        surface.pipeline_builder().pipeline(
            &self.backbuffer,
            &pipeline_state,
            |pipeline, mut shading_gate| {
                let queue = Module::from_existing("Draw")
                    .instance_variable_get("@queue")
                    .try_convert_to::<self::ruby::DrawQueue>();
                if let Ok(mut queue) = queue {
                    let commands = AsMut::<Vec<DrawCommand>>::as_mut(&mut queue);

                    SpriteProgram {
                        base: sprite_base,
                        program: sprite_program,
                        tess,
                        object: SpriteData::Commands(commands),
                    }
                    .render(&pipeline, &mut shading_gate);

                    PrimitiveProgram {
                        program: primitive_program,
                        tess,
                    }
                    .render(&mut shading_gate, commands);

                    TextProgram {
                        sprite: sprite_base,
                        text: text_base,
                        program: sprite_program,
                        tess,
                    }
                    .render(&pipeline, &mut shading_gate);
                }
            },
        );

        surface.swap_buffers();
    }
}

impl<'a> EngineModule for DrawModule<'a> {
    fn init(&mut self) {
        let mut module = Module::new("Draw");
        module.define_nested_class("DrawQueue", None);
        module.instance_variable_set("@queue", self::ruby::DrawQueue::new());

        module.def_self("arc!", self::ruby::draw_arc);
        module.def_self("circle!", self::ruby::draw_circle);
        module.def_self("line!", self::ruby::draw_line);
        module.def_self("rect!", self::ruby::draw_rectangle);
        module.def_self("rectangle!", self::ruby::draw_rectangle);
        module.def_self("sprite!", self::ruby::draw_sprite);
        module.def_self("text!", self::ruby::draw_text);
    }

    fn pre_update(&mut self) {
        self.clear_commands();

        let mut key_events = Vec::new();
        for event in self.surface.poll_events() {
            match event {
                WindowEvent::Close => {
                    panic!("Interrupt requested");
                }
                WindowEvent::Key(key, _, action, _) => {
                    key_events.push((key, action));
                }
                _ => {}
            }
        }
        for (key, action) in key_events {
            self.handle_key_event(key, action);
        }
    }

    fn post_update(&mut self) {
        self.prepare_render();
        self.render();
    }
}
