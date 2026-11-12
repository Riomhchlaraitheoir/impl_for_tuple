use std::fmt::Debug;
use impl_for_tuple::impl_for_tuple;

pub trait Validator<T: ?Sized>: Sized {
    type Error: Debug;
    fn validate(&self, value: &T) -> Result<(), Self::Error>;
}

impl_for_tuple! {
    impl<T: Tuple<{Self::Length}>> Validator<T> for Tuple<{1..=4}> where Self::Element<{i = ..}>: Validator<T::Element<{i}>> {
        type Error = Self::Mapped<V, Option<V::Error>>;
        fn validate(&self, value: &T) -> Result<(), Self::Error> {
            let mut valid = true;
            #[unroll_loop]
            let error = for i in 0..Self::Length {
                let error = self.[i].validate(&value.[i]).err();
                if error.is_some() {
                    valid = false;
                }
                error
            };
            if valid {
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

fn main() {

}