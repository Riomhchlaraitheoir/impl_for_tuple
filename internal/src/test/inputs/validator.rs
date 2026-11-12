impl<T: Tuple<{Self::Length}>> Validator<T> for Tuple<{0..=4}> where Self::Element<{i = ..}>: Validator<T::Element<{i}>> {
    type Error = Self::Mapped<V, Option<V::Error>>;
    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        let mut valid = true;
        #[unroll_loop]
        let error = for i in 0..Self::Length {
            let error = self.[i].validate(&value.[i]).err();
            if error.is_some() {
                valid = false;
            }
        };
        if valid {
            Ok(())
        } else {
            Err(error)
        }
    }
}