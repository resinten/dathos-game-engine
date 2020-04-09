use super::{DrawCommand, ObjectGeometry, SpritesheetLoadRequest, SpritesheetSlice};
use crate::ext::{AnyNumber, HashExt, RotationExt};
use crate::modules::core::{ColorData, VectorData};
use glyph_brush::{HorizontalAlign, VerticalAlign};
use nalgebra::Vector2;
use rutie::{Hash, Module, NilClass, Object, RString, Symbol, VerifiedObject};
use std::f32::consts::PI;
use std::path::PathBuf;

wrappable_struct!(DrawQueueInner, DrawQueueWrapper, DRAW_QUEUE_WRAPPER);

module!(Draw);

class!(DrawQueue);

pub struct DrawQueueInner {
    pub queue: Vec<DrawCommand>,
    pub pending_fonts: Vec<(String, PathBuf)>,
    pub pending_spritesheets: Vec<SpritesheetLoadRequest>,
    pub pending_sprites: Vec<SpritesheetSlice>,
}

impl Draw {
    fn load_font(&mut self, name: String, path: PathBuf) {
        let mut queue = self.instance_variable_get("@queue");
        let queue_inner = queue.get_data_mut(&*DRAW_QUEUE_WRAPPER);
        queue_inner.pending_fonts.push((name, path));
    }

    fn load_spritesheet(&mut self, name: String, path: PathBuf) {
        let mut queue = self.instance_variable_get("@queue");
        let queue_inner = queue.get_data_mut(&*DRAW_QUEUE_WRAPPER);
        queue_inner
            .pending_spritesheets
            .push(SpritesheetLoadRequest { name, path });
    }

    fn create_sprite(
        &mut self,
        name: String,
        spritesheet: String,
        offset: Vector2<f32>,
        size: Vector2<f32>,
    ) {
        let mut queue = self.instance_variable_get("@queue");
        let queue_inner = queue.get_data_mut(&*DRAW_QUEUE_WRAPPER);
        queue_inner.pending_sprites.push(SpritesheetSlice {
            name,
            spritesheet,
            offset,
            size,
        });
    }

    fn draw(&mut self, command: DrawCommand) {
        let mut queue = self.instance_variable_get("@queue");
        let queue_inner = queue.get_data_mut(&*DRAW_QUEUE_WRAPPER);
        queue_inner.queue.push(command);
    }
}

#[rustfmt::skip]
methods!(
    Draw,
    _itself,

    fn load_font(name: Symbol, path: RString) -> NilClass {
        _itself.load_font(name.unwrap().to_string(), From::from(path.unwrap().to_string()));
        NilClass::new()
    }

    fn load_spritesheet(name: Symbol, path: RString) -> NilClass {
        _itself.load_spritesheet(name.unwrap().to_string(), From::from(path.unwrap().to_string()));
        NilClass::new()
    }

    fn create_sprite(
        name: Symbol,
        spritesheet: Symbol,
        offset: VectorData,
        size: VectorData
    ) -> NilClass {
        _itself.create_sprite(
            name.unwrap().to_string(),
            spritesheet.unwrap().to_string(),
            offset.unwrap().into(),
            size.unwrap().into(),
        );
        NilClass::new()
    }

    fn draw_arc(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Arc {
            depth: options.get_num("depth").unwrap_or(1.0),
            color: options.get_as::<ColorData>("color")
                .map(|c| c.into())
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),
            radius: options.get_num("radius").unwrap_or(0.5),
            thickness: options.get_num("thickness").unwrap_or(0.5),
            from: options.get_num("from").unwrap_or(0.0).normalize(),
            to: options.get_num("to").unwrap_or(2.0 * PI).normalize(),
            position: options.get_as::<VectorData>("position")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
        });
        NilClass::new()
    }

    fn draw_circle(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Circle {
            depth: options.get_num("depth").unwrap_or(1.0),
            color: options.get_as::<ColorData>("color")
                .map(|c| c.into())
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),
            radius: options.get_num("radius").unwrap_or(1.0),
            position: options.get_as::<VectorData>("position")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
        });
        NilClass::new()
    }

    fn draw_line(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Line {
            depth: options.get_num("depth").unwrap_or(1.0),
            color: options.get_as::<ColorData>("color")
                .map(|c| c.into())
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),
            thickness: options.get_num("thickness").unwrap_or(1.0),
            from: options.get_as::<VectorData>("from")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
            to: options.get_as::<VectorData>("to")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
        });
        NilClass::new()
    }

    fn draw_rectangle(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Rectangle {
            depth: options.get_num("depth").unwrap_or(1.0),
            color: options.get_as::<ColorData>("color")
                .map(|c| c.into())
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),
            width: options.get_num("width").unwrap_or(1.0),
            height: options.get_num("height").unwrap_or(1.0),
            position: options.get_as::<VectorData>("position")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
            rotation: options.get_num("rotation").unwrap_or(0.0),
        });
        NilClass::new()
    }

    fn draw_sprite(options: Hash) -> NilClass {
        let options = options.unwrap();
        let sprite = options.get_as::<Symbol>("sprite").map(|s| s.to_string());
        let default_geometry = ObjectGeometry::default();
        if let Some(sprite) = sprite {
            _itself.draw(DrawCommand::Sprite {
                sprite,
                geometry: ObjectGeometry {
                    brighten: options
                        .get_as::<ColorData>("brighten")
                        .map(Into::into)
                        .unwrap_or(default_geometry.brighten),
                    darken: options
                        .get_as::<ColorData>("darken")
                        .map(Into::into)
                        .unwrap_or(default_geometry.darken),
                    depth: options.get_num("depth").unwrap_or(default_geometry.depth),
                    desaturation: options
                        .get_num("desaturation")
                        .unwrap_or(default_geometry.desaturation),
                    origin: options
                        .get_as::<VectorData>("origin")
                        .map(Into::into)
                        .unwrap_or(default_geometry.origin),
                    position: options
                        .get_as::<VectorData>("position")
                        .map(Into::into)
                        .unwrap_or(default_geometry.position),
                    rotation: options.get_num("rotation").unwrap_or(default_geometry.rotation),
                    scale: options
                        .get_as::<VectorData>("scale")
                        .map(Into::into)
                        .unwrap_or(default_geometry.scale),
                },
            });
        }
        NilClass::new()
    }

    fn draw_text(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Text {
            font: options.get_as::<Symbol>("font").map(|f| f.to_string()),
            depth: options.get_num("depth").unwrap_or(1.0),
            color: options.get_as::<ColorData>("color")
                .map(|c| c.into())
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),
            position: options.get_as::<VectorData>("position")
                .map(Into::<Vector2<f32>>::into)
                .unwrap_or_else(|| Vector2::new(0.0, 0.0)),
            size: options.get_as::<AnyNumber>("size")
                .map(|v| v.to_f32())
                .unwrap_or(1.0),
            text: options.get_as::<RString>("text")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string()),
            h_align: options.get_as::<Symbol>("halign").and_then(|h| {
                match h.to_string().to_lowercase().as_str() {
                    "left" => Some(HorizontalAlign::Left),
                    "center" => Some(HorizontalAlign::Center),
                    "middle" => Some(HorizontalAlign::Center),
                    "right" => Some(HorizontalAlign::Right),
                    _ => None,
                }
            }).unwrap_or(HorizontalAlign::Center),
            v_align: options.get_as::<Symbol>("valign").and_then(|v| {
                match v.to_string().to_lowercase().as_str() {
                    "top" => Some(VerticalAlign::Top),
                    "center" => Some(VerticalAlign::Center),
                    "middle" => Some(VerticalAlign::Center),
                    "bottom" => Some(VerticalAlign::Bottom),
                    _ => None,
                }
            }).unwrap_or(VerticalAlign::Center),
        });
        NilClass::new()
    }
);

impl DrawQueue {
    pub fn new() -> Self {
        Module::from_existing("Draw")
            .get_nested_class("DrawQueue")
            .wrap_data(
                DrawQueueInner {
                    queue: Vec::new(),
                    pending_fonts: Vec::new(),
                    pending_spritesheets: Vec::new(),
                    pending_sprites: Vec::new(),
                },
                &*DRAW_QUEUE_WRAPPER,
            )
    }
}

impl AsMut<Vec<DrawCommand>> for DrawQueue {
    fn as_mut(&mut self) -> &mut Vec<DrawCommand> {
        &mut self.get_data_mut(&*DRAW_QUEUE_WRAPPER).queue
    }
}

impl AsMut<Vec<(String, PathBuf)>> for DrawQueue {
    fn as_mut(&mut self) -> &mut Vec<(String, PathBuf)> {
        &mut self.get_data_mut(&*DRAW_QUEUE_WRAPPER).pending_fonts
    }
}

impl AsMut<Vec<SpritesheetLoadRequest>> for DrawQueue {
    fn as_mut(&mut self) -> &mut Vec<SpritesheetLoadRequest> {
        &mut self.get_data_mut(&*DRAW_QUEUE_WRAPPER).pending_spritesheets
    }
}

impl AsMut<Vec<SpritesheetSlice>> for DrawQueue {
    fn as_mut(&mut self) -> &mut Vec<SpritesheetSlice> {
        &mut self.get_data_mut(&*DRAW_QUEUE_WRAPPER).pending_sprites
    }
}

impl VerifiedObject for DrawQueue {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Module::from_existing("Draw").get_nested_class("DrawQueue")
    }

    fn error_message() -> &'static str { "Object is not instance of type DrawQueue" }
}
