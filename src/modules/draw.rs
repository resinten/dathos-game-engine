use self::primitive::{PrimitiveProgram, PrimitiveShaderInterface};
use self::sprite::{
    SpriteData, SpriteProgram, SpriteProgramBase, SpriteShaderInterface, SpritesheetLoader,
};
use self::text::{TextProgram, TextProgramBase};
use super::core::INPUT_WRAPPER;
use super::{EngineModule, GameState};
use glyph_brush::{HorizontalAlign, VerticalAlign};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::pixel::NormRGBA8UI;
use luminance::shader::program::{Program, ProgramError};
use luminance::tess::{Mode, Tess, TessBuilder, TessError};
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Texture, Wrap};
use luminance_glfw::{
    Action, GlfwSurface, GlfwSurfaceError, Key, Surface, WindowDim, WindowEvent, WindowOpt,
};
use nalgebra::{Vector2, Vector4};
use rutie::{Module, Object};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

mod primitive;
mod ruby;
mod sprite;
mod text;

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

#[derive(Clone, Debug)]
pub struct SpritesheetLoadRequest {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SpritesheetSlice {
    name: String,
    spritesheet: String,
    offset: Vector2<f32>,
    size: Vector2<f32>,
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

    load_requests: Sender<(String, PathBuf)>,
    loaded_textures: Receiver<(String, Vector2<u32>, Vec<u8>)>,

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
    Sprite {
        sprite: String,
        geometry: ObjectGeometry,
    },
    Text {
        depth: f32,
        color: [f32; 4],
        position: Vector2<f32>,
        size: f32,
        text: String,
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
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
    pub fn build<G>(game_state: &G) -> Result<Self, BuildError>
    where
        G: GameState,
    {
        let options = game_state.window_options();
        let mut surface = GlfwSurface::new(
            WindowDim::Windowed(options.width, options.height),
            &options.title,
            WindowOpt::default(),
        )?;
        let backbuffer = surface.back_buffer().unwrap();

        let text_base = TextProgramBase::new(&mut surface, game_state);
        let tess = TessBuilder::new(&mut surface)
            .set_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()?;

        let (load_requests, loaded_textures, spritesheet_loader) = SpritesheetLoader::build();
        std::thread::spawn(|| spritesheet_loader.run());

        Ok(DrawModule {
            surface,
            backbuffer,

            load_requests,
            loaded_textures,

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

    fn handle_spritesheet_loading(&mut self) {
        let queue = Module::from_existing("Draw")
            .instance_variable_get("@queue")
            .try_convert_to::<self::ruby::DrawQueue>();
        if let Ok(mut queue) = queue {
            {
                let pending_spritesheets = AsMut::<Vec<SpritesheetLoadRequest>>::as_mut(&mut queue);
                pending_spritesheets.drain(..).for_each(|ps| {
                    let _ = self.load_requests.send((ps.name, ps.path));
                });
            }
            {
                let pending_sprites = AsMut::<Vec<SpritesheetSlice>>::as_mut(&mut queue);
                pending_sprites.drain(..).for_each(|ps| {
                    self.sprite_base
                        .sprites
                        .insert(ps.name, (ps.spritesheet, ps.offset, ps.size));
                });
            }
        }

        while let Ok((name, size, texels)) = self.loaded_textures.try_recv() {
            let texture =
                Texture::<Dim2, NormRGBA8UI>::new(&mut self.surface, [size.x, size.y], 0, SAMPLER)
                    .unwrap();
            texture.upload_raw(GenMipmaps::No, &texels).unwrap();
            self.sprite_base.spritesheets.insert(name, texture);
        }
    }

    fn prepare_render<G>(&mut self, game_state: &G)
    where
        G: GameState,
    {
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
            .prepare_render(&mut self.surface, game_state, commands);
        }
    }

    fn render<G>(&mut self, game_state: &G)
    where
        G: GameState,
    {
        let surface = &mut self.surface;
        let primitive_program = &self.primitive_program;
        let sprite_base = &self.sprite_base;
        let sprite_program = &self.sprite_program;
        let text_base = &mut self.text_base;
        let tess = &self.tess;

        let pipeline_state = PipelineState::new()
            .enable_clear_color(true)
            .set_clear_color([1.0, 1.0, 1.0, 1.0]);

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
                    .render(&pipeline, &mut shading_gate, game_state);

                    PrimitiveProgram {
                        program: primitive_program,
                        tess,
                    }
                    .render(&mut shading_gate, game_state, commands);

                    TextProgram {
                        sprite: sprite_base,
                        text: text_base,
                        program: sprite_program,
                        tess,
                    }
                    .render(&pipeline, &mut shading_gate, game_state);
                }
            },
        );

        surface.swap_buffers();
    }
}

impl<'a, G> EngineModule<G> for DrawModule<'a>
where
    G: GameState,
{
    fn init(&mut self, _: &mut G) {
        let mut module = Module::new("Draw");
        module.define_nested_class("DrawQueue", None);
        module.instance_variable_set("@queue", self::ruby::DrawQueue::new());

        module.def_self("load_spritesheet", self::ruby::load_spritesheet);
        module.def_self("create_sprite", self::ruby::create_sprite);

        module.def_self("arc!", self::ruby::draw_arc);
        module.def_self("circle!", self::ruby::draw_circle);
        module.def_self("line!", self::ruby::draw_line);
        module.def_self("rect!", self::ruby::draw_rectangle);
        module.def_self("rectangle!", self::ruby::draw_rectangle);
        module.def_self("sprite!", self::ruby::draw_sprite);
        module.def_self("text!", self::ruby::draw_text);
    }

    fn pre_update(&mut self, _: &mut G) {
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

    fn post_update(&mut self, game_state: &mut G) {
        self.handle_spritesheet_loading();
        self.prepare_render(game_state);
        self.render(game_state);
    }
}
