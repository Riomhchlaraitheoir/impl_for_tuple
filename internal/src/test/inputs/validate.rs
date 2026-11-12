impl Validate for Tuple<{ ..=4 }> where Self::Element<{..}>: Validate {
    type Valid = Self::Mapped<T, T::Valid>;
    fn validate(&self) -> Result<Self::Valid, String> {
        #[unroll_loop]
        for i in 0..Self::Length {
            self.[i].validate()?
        }
    }
}