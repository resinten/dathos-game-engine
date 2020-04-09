use super::{DrawCommand, ObjectGeometry, SCREEN_HEIGHT, SCREEN_WIDTH};
use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::depth_test::DepthComparison;
use luminance::pipeline::{BoundTexture, Pipeline, ShadingGate};
use luminance::pixel::{Depth32F, NormRGBA8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::Tess;
use luminance::texture::{Dim2, Texture};
use luminance_derive::UniformInterface;
use nalgebra::Vector2;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(UniformInterface)]
pub struct SpriteShaderInterface {
    pub screen_size: Uniform<[f32; 2]>,
    pub hidpi_factor: Uniform<f32>,
    pub image_size: Uniform<[f32; 2]>,
    pub subimage_offset: Uniform<[f32; 2]>,
    pub subimage_size: Uniform<[f32; 2]>,

    pub origin: Uniform<[f32; 2]>,
    pub position: Uniform<[f32; 2]>,
    pub rotation: Uniform<f32>,
    pub scale: Uniform<[f32; 2]>,

    pub depth: Uniform<f32>,
    pub image: Uniform<&'static BoundTexture<'static, Dim2, NormUnsigned>>,
    // pub depth_buffer: Uniform<&'static BoundTexture<'static, Dim2, Floating>>,
    // pub use_depth_buffer: Uniform<bool>,
    pub brighten: Uniform<[f32; 4]>,
    pub darken: Uniform<[f32; 4]>,
    pub desaturation: Uniform<f32>,
}

pub struct SpritesheetLoader {
    load_requests: Receiver<(String, PathBuf)>,
    loaded_textures: Sender<(String, Vector2<u32>, Vec<u8>)>,
}

pub struct SpriteProgramBase {
    pub spritesheets: BTreeMap<String, Texture<Dim2, NormRGBA8UI>>,
    pub sprites: BTreeMap<String, (String, Vector2<f32>, Vector2<f32>)>,
}

pub struct SpriteProgram<'a> {
    pub base: &'a SpriteProgramBase,
    pub program: &'a Program<(), (), SpriteShaderInterface>,
    pub tess: &'a Tess,
    pub object: SpriteData<'a>,
}

#[derive(Clone, Copy)]
pub enum SpriteData<'a> {
    Commands(&'a Vec<DrawCommand>),
    Override {
        texture: &'a Texture<Dim2, NormRGBA8UI>,
        depth_buffer: Option<&'a Texture<Dim2, Depth32F>>,
        instances: &'a Vec<DrawInstance>,
    },
}

#[derive(Clone)]
pub struct DrawInstance {
    pub offset: Vector2<f32>,
    pub size: Vector2<f32>,
    pub geometry: ObjectGeometry,
}

impl SpritesheetLoader {
    pub fn build() -> (
        Sender<(String, PathBuf)>,
        Receiver<(String, Vector2<u32>, Vec<u8>)>,
        Self,
    ) {
        let (load_requests_sender, load_requests_receiver) = channel();
        let (loaded_textures_sender, loaded_textures_receiver) = channel();
        (
            load_requests_sender,
            loaded_textures_receiver,
            SpritesheetLoader {
                load_requests: load_requests_receiver,
                loaded_textures: loaded_textures_sender,
            },
        )
    }

    pub fn run(self) {
        loop {
            for (name, path) in self.load_requests.recv() {
                println!("Loading sprite {}", name);
                let buffer: Option<(Vector2<u32>, Vec<u8>)> = try {
                    let buffer = image::load(
                        BufReader::new(File::open(path).ok()?),
                        image::ImageFormat::Png,
                    )
                    .ok()?
                    .flipv()
                    .to_rgba();
                    let (width, height) = buffer.dimensions();
                    ([width, height].into(), buffer.into_raw())
                };
                if let Some((dimensions, buffer)) = buffer {
                    let result = self.loaded_textures.send((name, dimensions, buffer));
                    if let Err(_) = result {
                        println!("Failed to send a loaded spritesheet to the draw module");
                    }
                } else {
                    println!("Failed to load an image for a spritesheet");
                }
            }
        }
    }
}

impl SpriteProgramBase {
    pub fn new() -> Self {
        SpriteProgramBase {
            spritesheets: BTreeMap::new(),
            sprites: BTreeMap::new(),
        }
    }
}

impl<'a> SpriteProgram<'a> {
    pub fn render<C>(&mut self, pipeline: &Pipeline, shading_gate: &mut ShadingGate<C>)
    where
        C: GraphicsContext,
    {
        let instances_list = match self.object {
            SpriteData::Commands(commands) => {
                let mut grouped_commands = BTreeMap::new();
                for command in commands {
                    if let DrawCommand::Sprite { sprite, geometry } = command {
                        if let Some((spritesheet, offset, size)) = self.base.sprites.get(sprite) {
                            grouped_commands
                                .entry(spritesheet)
                                .or_insert_with(|| Vec::new())
                                .push(DrawInstance {
                                    offset: *offset,
                                    size: *size,
                                    geometry: geometry.clone(),
                                });
                        } else {
                            println!("Could not find wprite {:?}", sprite);
                        }
                    }
                }
                grouped_commands
                    .into_iter()
                    .filter_map(|(spritesheet, instances)| {
                        Some((self.base.spritesheets.get(spritesheet)?, None, instances))
                    })
                    .collect::<Vec<_>>()
            }
            SpriteData::Override {
                texture,
                depth_buffer,
                instances,
            } => vec![(texture, depth_buffer, (*instances).clone())],
        };
        for (texture, depth_buffer, instances) in instances_list {
            self.render_instances(pipeline, shading_gate, texture, depth_buffer, instances);
        }
    }

    fn render_instances<C>(
        &mut self,
        pipeline: &Pipeline,
        shading_gate: &mut ShadingGate<C>,
        texture: &Texture<Dim2, NormRGBA8UI>,
        depth_buffer: Option<&Texture<Dim2, Depth32F>>,
        instances: Vec<DrawInstance>,
    ) where
        C: GraphicsContext,
    {
        let render_state: RenderState = Default::default();
        let render_state = render_state
            .set_blending(Some((
                Equation::Additive,
                Factor::SrcAlpha,
                Factor::SrcAlphaComplement,
            )))
            .set_depth_test(Some(DepthComparison::Always));

        let image_size = texture.size();
        let image = pipeline.bind_texture(texture);
        let depth_image = depth_buffer.map(|db| pipeline.bind_texture(db));
        shading_gate.shade(&self.program, |interface, mut render_gate| {
            // if let Some(depth_image) = depth_image {
            //     interface.depth_buffer.update(&depth_image);
            //     interface.use_depth_buffer.update(true);
            // } else {
            //     interface.use_depth_buffer.update(false);
            // }

            interface
                .screen_size
                .update([SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]);
            interface.hidpi_factor.update(2.0);
            interface
                .image_size
                .update([image_size[0] as f32, image_size[1] as f32]);

            for DrawInstance {
                offset,
                size,
                geometry,
            } in instances
            {
                interface.subimage_offset.update(offset.into());
                interface.subimage_size.update(size.into());

                // Tweak because everything gets rendered upside down
                let mut origin = geometry.origin;
                origin.y = 1.0 - origin.y;

                interface.depth.update(geometry.depth);
                interface.origin.update(origin.into());
                interface.position.update(geometry.position.into());
                interface.rotation.update(geometry.rotation);
                interface.scale.update(geometry.scale.into());

                interface.image.update(&image);
                interface.brighten.update(geometry.brighten.into());
                interface.darken.update(geometry.darken.into());
                interface.desaturation.update(geometry.desaturation);

                render_gate.render(&render_state, |mut tess_gate| {
                    tess_gate.render(self.tess);
                });
            }
        });
    }
}
