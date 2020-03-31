use std::f32::consts::PI;

const PI_2: f32 = 2.0 * PI;
const FRAC_PI_8: f32 = PI / 8.0;

pub trait RotationExt {
    fn angular_distance(&self, other: &Self) -> f32;
    fn clockwise_distance(&self, other: &Self) -> f32;
    fn counter_clockwise_distance(&self, other: &Self) -> f32;
    fn normalize(self) -> Self;
    fn octodirectional(self) -> Self;
    fn shortest_rotation_direction(&self, other: &Self) -> RotationDirection;
}

#[derive(PartialEq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl RotationExt for f32 {
    fn angular_distance(&self, other: &Self) -> f32 {
        self.clockwise_distance(other)
            .min(self.counter_clockwise_distance(other))
    }

    fn clockwise_distance(&self, other: &Self) -> f32 { (self - other).normalize() }

    fn counter_clockwise_distance(&self, other: &Self) -> f32 { other.clockwise_distance(self) }

    fn normalize(mut self) -> Self {
        while self < 0.0 {
            self += PI_2;
        }
        self % PI_2
    }

    fn octodirectional(self) -> Self {
        let r = self.normalize();
        if r > 15.0 * FRAC_PI_8 || r <= FRAC_PI_8 {
            0.0
        } else if r > FRAC_PI_8 && r <= 3.0 * FRAC_PI_8 {
            PI / 4.0
        } else if r > 3.0 * FRAC_PI_8 && r <= 5.0 * FRAC_PI_8 {
            PI / 2.0
        } else if r > 5.0 * FRAC_PI_8 && r <= 7.0 * FRAC_PI_8 {
            3.0 * PI / 4.0
        } else if r > 7.0 * FRAC_PI_8 && r <= 9.0 * FRAC_PI_8 {
            PI
        } else if r > 9.0 * FRAC_PI_8 && r <= 11.0 * FRAC_PI_8 {
            5.0 * PI / 4.0
        } else if r > 11.0 * FRAC_PI_8 && r <= 13.0 * FRAC_PI_8 {
            3.0 * PI / 2.0
        } else {
            7.0 * PI / 4.0
        }
    }

    fn shortest_rotation_direction(&self, other: &Self) -> RotationDirection {
        let from = self.normalize();
        let to = other.normalize();

        if from.counter_clockwise_distance(&to) > PI {
            RotationDirection::Clockwise
        } else {
            RotationDirection::CounterClockwise
        }
    }
}
