use crate::ext::{AnyNumber, RotationDirection, RotationExt};
use rutie::{Float, Module, Object};
use std::f32::consts::PI;

module!(Rotation);

#[rustfmt::skip]
methods!(
    Rotation,
    _itself,

    fn angular_distance(lhs: AnyNumber, rhs: AnyNumber) -> Float {
        Float::new(lhs.unwrap().to_f32().angular_distance(&rhs.unwrap().to_f32()) as f64)
    }

    fn radians_to_degrees(radians: AnyNumber) -> Float {
        Float::new((180.0 * radians.unwrap().to_f32() / PI) as f64)
    }

    fn degrees_to_radians(degrees: AnyNumber) -> Float {
        Float::new((PI * degrees.unwrap().to_f32() / 180.0) as f64)
    }

    fn octodirectional(angle: AnyNumber) -> Float {
        Float::new(angle.unwrap().to_f32().octodirectional() as f64)
    }

    fn rotate_toward(
        from: AnyNumber,
        to: AnyNumber,
        max_delta: AnyNumber
    ) -> Float {
        let (from, to, max_delta) = (
            from.unwrap().to_f32(),
            to.unwrap().to_f32(),
            max_delta.unwrap().to_f32(),
        );
        let direction = from.shortest_rotation_direction(&to);
        let distance = from.angular_distance(&to);
        let scalar = match direction {
            RotationDirection::Clockwise => -1.0,
            RotationDirection::CounterClockwise => 1.0,
        };
        Float::new(if distance <= max_delta {
            to
        } else {
            from + scalar * max_delta
        }.normalize() as f64)
    }
);

pub fn add_rotation_module() {
    let mut module = Module::new("Rotation");
    module.def_self("distance", angular_distance);
    module.def_self("radians_to_degrees", radians_to_degrees);
    module.def_self("degrees_to_radians", degrees_to_radians);
    module.def_self("octodirectional", octodirectional);
    module.def_self("rotate_toward", rotate_toward);
}
