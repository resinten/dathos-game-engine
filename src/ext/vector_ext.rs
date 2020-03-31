use nalgebra::Vector2;

pub trait VectorExt {
    fn from_rotation(rotation: f32) -> Self;

    fn project_onto(self, other: &Self) -> Self;
    fn distance_from_line(self, from: &Self, to: &Self) -> f32;
    fn round(&mut self);
    fn rounded(self) -> Self;
    fn to_rotation(&self) -> f32;
}

impl VectorExt for Vector2<f32> {
    fn from_rotation(rotation: f32) -> Self {
        Vector2::new(rotation.cos(), rotation.sin())
    }

    fn project_onto(self, other: &Self) -> Self {
        let other_norm = other.normalize();
        self * other_norm.dot(&self)
    }

    // https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Line_defined_by_two_points
    fn distance_from_line(self, from: &Self, to: &Self) -> f32 {
        let numerator = ((to.y - from.y) * self.x - (to.x - from.x) * self.y + to.x * from.y
            - to.y * from.x)
            .abs();
        let denominator = ((to.y - from.y).powi(2) + (to.x - from.x).powi(2)).sqrt();
        numerator / denominator
    }

    fn round(&mut self) {
        self.x = self.x.round();
        self.y = self.y.round();
    }

    fn rounded(mut self) -> Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self
    }

    fn to_rotation(&self) -> f32 {
        self.y.atan2(self.x)
    }
}
