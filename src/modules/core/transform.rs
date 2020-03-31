use super::vector::VectorData;
use crate::ext::{AnyNumber, RotationDirection, RotationExt};
use nalgebra::Vector2;
use rutie::{Class, Float, Object, VerifiedObject};

wrappable_struct!(TransformInner, TransformWrapper, TRANSFORM_WRAPPER);

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vector2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
}

class!(TransformData);

pub struct TransformInner {
    pub inner: Transform,
}

impl From<Transform> for TransformData {
    fn from(t: Transform) -> Self {
        Class::from_existing("Transform")
            .wrap_data(TransformInner { inner: t }, &*TRANSFORM_WRAPPER)
    }
}

impl Into<Transform> for TransformData {
    fn into(self) -> Transform { self.get_data(&*TRANSFORM_WRAPPER).inner }
}

#[rustfmt::skip]
methods!(
    TransformData,
    _itself,

    fn new_transform(
        position: VectorData,
        rotation: AnyNumber,
        scale: VectorData
    ) -> TransformData {
        From::from(Transform {
            position: position.unwrap().into(),
            rotation: rotation.unwrap().to_f32(),
            scale: scale.unwrap().into(),
        })
    }

    fn get_position() -> VectorData {
        From::from(Into::<Transform>::into(_itself).position)
    }

    fn get_rotation() -> Float {
        Float::new(Into::<Transform>::into(_itself).rotation as f64)
    }

    fn get_scale() -> VectorData {
        From::from(Into::<Transform>::into(_itself).scale)
    }

    fn set_position(p: VectorData) -> VectorData {
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        transform.inner.position = p.unwrap().into();
        From::from(transform.inner.position)
    }

    fn set_rotation(r: AnyNumber) -> Float {
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        transform.inner.rotation = r.unwrap().to_f32();
        Float::new(transform.inner.rotation as f64)
    }

    fn set_scale(s: VectorData) -> VectorData {
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        transform.inner.scale = s.unwrap().into();
        From::from(transform.inner.scale)
    }

    fn move_transform(v: VectorData) -> VectorData {
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        transform.inner.position += Into::<Vector2<f32>>::into(v.unwrap());
        From::from(transform.inner.position)
    }

    fn move_toward(v: VectorData, max_delta: AnyNumber) -> VectorData {
        let max_delta = max_delta.unwrap().to_f32();
        let destination = Into::<Vector2<f32>>::into(v.unwrap());
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        if (destination - transform.inner.position).magnitude() < max_delta {
            transform.inner.position = destination;
        } else {
            transform.inner.position +=
                max_delta * (destination - transform.inner.position).normalize();
        }
        From::from(transform.inner.position)
    }

    fn rotate_transform(r: AnyNumber) -> Float {
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        transform.inner.rotation = (transform.inner.rotation + r.unwrap().to_f32()).normalize();
        Float::new(transform.inner.rotation as f64)
    }

    fn rotate_toward(r: AnyNumber, max_delta: AnyNumber) -> Float {
        let r = r.unwrap().to_f32().normalize();
        let max_delta = max_delta.unwrap().to_f32();
        let mut transform = _itself.get_data_mut(&*TRANSFORM_WRAPPER);
        let (rotation_direction, rotation_distance) = rotation_state(transform.inner.rotation, r);
        transform.inner.rotation = if rotation_distance <= max_delta {
            r
        } else {
            (transform.inner.rotation + match rotation_direction {
                RotationDirection::Clockwise => -max_delta,
                RotationDirection::CounterClockwise => max_delta,
            }).normalize()
        };
        Float::new(transform.inner.rotation as f64)
    }
);

impl VerifiedObject for TransformData {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::from_existing("Transform")
    }

    fn error_message() -> &'static str { "Object is not type of class Transform" }
}

fn rotation_state(rotation: f32, desired_facing: f32) -> (RotationDirection, f32) {
    let direction = rotation.shortest_rotation_direction(&desired_facing);
    let distance = match direction {
        RotationDirection::Clockwise => rotation.clockwise_distance(&desired_facing),
        RotationDirection::CounterClockwise => rotation.counter_clockwise_distance(&desired_facing),
    };
    (direction, distance)
}

pub fn add_transform_class() {
    let mut class = Class::new("Transform", None);
    class.def_self("new", new_transform);

    class.def("position", get_position);
    class.def("rotation", get_rotation);
    class.def("scale", get_scale);

    class.def("position=", set_position);
    class.def("rotation=", set_rotation);
    class.def("scale=", set_scale);

    class.def("move", move_transform);
    class.def("move_toward", move_toward);
    class.def("rotate", rotate_transform);
    class.def("rotate_toward", rotate_toward);
}
