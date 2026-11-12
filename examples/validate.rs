use impl_for_tuple::impl_for_tuple;

trait Validate {
    type Valid;
    fn validate(&self) -> Result<Self::Valid, String>;
}

impl_for_tuple! {
    impl Validate for Tuple where Self::Element<{..}>: Validate {
        type Valid = Self::Mapped<T, T::Valid>;
        fn validate(&self) -> Result<Self::Valid, String> {
            let valid = #[unroll_loop] for i in 0..Self::Length {
                self.[i].validate()?
            };
            Ok(valid)
        }
    }
}
impl Validate for bool {
    type Valid = ();
    fn validate(&self) -> Result<(), String> {
        if *self {
            Ok(())
        } else {
            Err("invalid".to_string())
        }
    }
}

fn main() {
    let tuple = (
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    );
    let result = tuple.validate();

    assert_eq!(result, Ok((
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
    )));
}