use crate::ext::HashExt;
use nalgebra::Vector4;
use rutie::{Class, Float, Hash, Object, VerifiedObject};

wrappable_struct!(ColorInner, ColorWrapper, COLOR_WRAPPER);

class!(ColorData);

pub struct ColorInner {
    pub color: [f32; 4],
}

impl From<[f32; 3]> for ColorData {
    fn from(color: [f32; 3]) -> Self { From::from([color[0], color[1], color[2], 0.0]) }
}

impl From<[f32; 4]> for ColorData {
    fn from(color: [f32; 4]) -> Self {
        Class::from_existing("Color").wrap_data(ColorInner { color }, &*COLOR_WRAPPER)
    }
}

impl Into<[f32; 4]> for ColorData {
    fn into(self) -> [f32; 4] { self.get_data(&*COLOR_WRAPPER).color }
}

impl Into<Vector4<f32>> for ColorData {
    fn into(self) -> Vector4<f32> { self.get_data(&*COLOR_WRAPPER).color.into() }
}

#[rustfmt::skip]
methods!(
    ColorData,
    _itself,

    fn new_color(options: Hash) -> ColorData {
        let options = options.unwrap();
        From::from([
            options.get_either_num("red", "r").unwrap_or(0.0),
            options.get_either_num("green", "g").unwrap_or(0.0),
            options.get_either_num("blue", "b").unwrap_or(0.0),
            options.get_either_num("alpha", "a").unwrap_or(1.0),
        ])
    }

    fn get_red() -> Float {
        Float::new(_itself.get_data(&*COLOR_WRAPPER).color[0] as f64)
    }

    fn get_green() -> Float {
        Float::new(_itself.get_data(&*COLOR_WRAPPER).color[1] as f64)
    }

    fn get_blue() -> Float {
        Float::new(_itself.get_data(&*COLOR_WRAPPER).color[2] as f64)
    }

    fn get_alpha() -> Float {
        Float::new(_itself.get_data(&*COLOR_WRAPPER).color[3] as f64)
    }
);

impl VerifiedObject for ColorData {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::from_existing("Color")
    }

    fn error_message() -> &'static str { "Object is not of type Color" }
}

pub fn add_color_class() {
    let mut class = Class::new("Color", None);
    class.def_self("new", new_color);

    class.def("r", get_red);
    class.def("g", get_green);
    class.def("b", get_blue);
    class.def("a", get_alpha);
    class.def("red", get_red);
    class.def("green", get_green);
    class.def("blue", get_blue);
    class.def("alpha", get_alpha);
}
