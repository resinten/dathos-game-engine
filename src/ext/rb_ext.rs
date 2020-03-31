use rutie::{Fixnum, Float, Hash, Integer, Object, Symbol, VerifiedObject};

pub trait HashExt {
    fn get_as<V>(&self, key: &str) -> Option<V>
    where
        V: VerifiedObject;
    fn get_either<V>(&self, key_1: &str, key_2: &str) -> Option<V>
    where
        V: VerifiedObject;
    fn get_num(&self, key: &str) -> Option<f32>;
    fn get_either_num(&self, key_1: &str, key_2: &str) -> Option<f32>;
}

class!(AnyNumber);

impl HashExt for Hash {
    fn get_as<V>(&self, key: &str) -> Option<V>
    where
        V: VerifiedObject,
    {
        self.at(&Symbol::new(key.as_ref()))
            .try_convert_to::<V>()
            .ok()
    }

    fn get_either<V>(&self, key_1: &str, key_2: &str) -> Option<V>
    where
        V: VerifiedObject,
    {
        self.get_as::<V>(key_1).or_else(|| self.get_as::<V>(key_2))
    }

    fn get_num(&self, key: &str) -> Option<f32> {
        self.get_as::<AnyNumber>(key).map(AnyNumber::to_f32)
    }

    fn get_either_num(&self, key_1: &str, key_2: &str) -> Option<f32> {
        self.get_either::<AnyNumber>(key_1, key_2)
            .map(AnyNumber::to_f32)
    }
}

impl AnyNumber {
    pub fn to_f32(self) -> f32 {
        if Float::is_correct_type(&self) {
            self.try_convert_to::<Float>().unwrap().to_f64() as f32
        } else if Integer::is_correct_type(&self) {
            self.try_convert_to::<Integer>().unwrap().to_i64() as f32
        } else if Fixnum::is_correct_type(&self) {
            self.try_convert_to::<Fixnum>().unwrap().to_i64() as f32
        } else {
            unreachable!("AnyNumber should be some number")
        }
    }
}

impl VerifiedObject for AnyNumber {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Float::is_correct_type(object)
            || Integer::is_correct_type(object)
            || Fixnum::is_correct_type(object)
    }

    fn error_message() -> &'static str { "Object is not instance of any number type" }
}
