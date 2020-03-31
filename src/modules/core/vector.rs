use crate::ext::{AnyNumber, VectorExt};
use nalgebra::Vector2;
use rutie::{Class, Float, Object, RString, VerifiedObject};

wrappable_struct!(VectorInner, VectorWrapper, VECTOR_WRAPPER);

class!(VectorData);

pub struct VectorInner {
    pub inner: Vector2<f32>,
}

#[rustfmt::skip]
methods!(
    VectorData,
    _itself,

    fn new_vector(x: AnyNumber, y: AnyNumber) -> VectorData {
        From::from(Vector2::new(x.unwrap().to_f32(), y.unwrap().to_f32()))
    }

    fn from_rotation(a: AnyNumber) -> VectorData {
        Vector2::from_rotation(a.unwrap().to_f32()).into()
    }
);

#[rustfmt::skip]
methods!(
    VectorData,
    _itself,

    fn project_onto(other: VectorData) -> VectorData {
        From::from({
            let other = Into::<Vector2<f32>>::into(other.unwrap());
            if other.magnitude() > 0.0 {
                Into::<Vector2<f32>>::into(_itself).project_onto(&other)
            } else {
                Vector2::new(0.0, 0.0)
            }
        })
    }

    fn scale_to(scale: AnyNumber) -> VectorData {
        From::from({
            let current = Into::<Vector2<f32>>::into(_itself);
            scale.unwrap().to_f32() * if current.magnitude() > 0.0 {
                current.normalize()
            } else {
                current
            }
        })
    }

    fn to_rotation() -> Float {
        Float::new(Into::<Vector2<f32>>::into(_itself).to_rotation() as f64)
    }

    fn to_magnitude() -> Float {
        Float::new(Into::<Vector2<f32>>::into(_itself).magnitude() as f64)
    }

    fn to_string() -> RString {
        RString::new_utf8(&format!("{:?}", Into::<Vector2<f32>>::into(_itself)))
    }
);

#[rustfmt::skip]
methods!(
    VectorData,
    _itself,

    fn get_x() -> Float {
        Float::new(_itself.get_data(&*VECTOR_WRAPPER).inner.x as f64)
    }

    fn get_y() -> Float {
        Float::new(_itself.get_data(&*VECTOR_WRAPPER).inner.y as f64)
    }

    fn operator_add(other: VectorData) -> VectorData {
        let self_inner = _itself.get_data(&*VECTOR_WRAPPER).inner;
        let other_inner = other.unwrap().get_data(&*VECTOR_WRAPPER).inner;
        let new_inner = VectorInner {
            inner: self_inner + other_inner
        };
        Class::from_existing("Vector").wrap_data(new_inner, &*VECTOR_WRAPPER)
    }

    fn operator_sub(other: VectorData) -> VectorData {
        let self_inner = _itself.get_data(&*VECTOR_WRAPPER).inner;
        let other_inner = other.unwrap().get_data(&*VECTOR_WRAPPER).inner;
        let new_inner = VectorInner {
            inner: self_inner - other_inner
        };
        Class::from_existing("Vector").wrap_data(new_inner, &*VECTOR_WRAPPER)
    }

    fn operator_mul(other: AnyNumber) -> VectorData {
        let other = other.unwrap().to_f32();
        let inner = _itself.get_data(&*VECTOR_WRAPPER).inner;
        let new_inner = VectorInner {
            inner: inner * other,
        };
        Class::from_existing("Vector").wrap_data(new_inner, &*VECTOR_WRAPPER)
    }

    fn operator_div(other: AnyNumber) -> VectorData {
        let other = other.unwrap().to_f32();
        let inner = _itself.get_data(&*VECTOR_WRAPPER).inner;
        let new_inner = VectorInner {
            inner: inner / other,
        };
        Class::from_existing("Vector").wrap_data(new_inner, &*VECTOR_WRAPPER)
    }
);

impl AsMut<Vector2<f32>> for VectorData {
    fn as_mut(&mut self) -> &mut Vector2<f32> { &mut self.get_data_mut(&*VECTOR_WRAPPER).inner }
}

impl From<Vector2<f32>> for VectorData {
    fn from(v: Vector2<f32>) -> Self {
        Class::from_existing("Vector").wrap_data(VectorInner { inner: v }, &*VECTOR_WRAPPER)
    }
}

impl Into<Vector2<f32>> for VectorData {
    fn into(self) -> Vector2<f32> { self.get_data(&*VECTOR_WRAPPER).inner }
}

impl VerifiedObject for VectorData {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::from_existing("Vector")
    }

    fn error_message() -> &'static str { "Object is not type of class Vector" }
}

pub fn add_vector_class() {
    let mut class = Class::new("Vector", None);
    class.def_self("new", new_vector);
    class.def_self("from_rotation", from_rotation);

    class.def("project_onto", project_onto);
    class.def("scale_to", scale_to);
    class.def("rotation", to_rotation);
    class.def("magnitude", to_magnitude);
    class.def("to_s", to_string);

    class.def("x", get_x);
    class.def("y", get_y);

    class.def("+", operator_add);
    class.def("-", operator_sub);
    class.def("*", operator_mul);
    class.def("/", operator_div);
}
