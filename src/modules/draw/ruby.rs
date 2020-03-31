use super::DrawCommand;
use crate::ext::{AnyNumber, HashExt, RotationExt};
use crate::modules::core::{ColorData, VectorData};
use nalgebra::Vector2;
use rutie::{Hash, Module, NilClass, Object, RString, VerifiedObject};
use std::f32::consts::PI;

wrappable_struct!(DrawQueueInner, DrawQueueWrapper, DRAW_QUEUE_WRAPPER);

module!(Draw);

class!(DrawQueue);

pub struct DrawQueueInner {
    pub queue: Vec<DrawCommand>,
}

impl Draw {
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
        unimplemented!("Not implemented yet");
    }

    fn draw_text(options: Hash) -> NilClass {
        let options = options.unwrap();
        _itself.draw(DrawCommand::Text {
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
        });
        NilClass::new()
    }
);

impl DrawQueue {
    pub fn new() -> Self {
        Module::from_existing("Draw")
            .get_nested_class("DrawQueue")
            .wrap_data(DrawQueueInner { queue: Vec::new() }, &*DRAW_QUEUE_WRAPPER)
    }
}

impl AsMut<Vec<DrawCommand>> for DrawQueue {
    fn as_mut(&mut self) -> &mut Vec<DrawCommand> {
        &mut self.get_data_mut(&*DRAW_QUEUE_WRAPPER).queue
    }
}

impl VerifiedObject for DrawQueue {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Module::from_existing("Draw").get_nested_class("DrawQueue")
    }

    fn error_message() -> &'static str { "Object is not instance of type DrawQueue" }
}
