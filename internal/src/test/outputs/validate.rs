impl Validate for () {
    type Valid = ();
    fn validate(&self) -> Result<Self::Valid, String> {
        ()
    }
}

impl<_0> Validate for (_0,)
where
    _0: Validate,
{
    type Valid = (_0::Valid,);
    fn validate(&self) -> Result<Self::Valid, String> {
        ({ self.0.validate()? },)
    }
}
impl<_0, _1> Validate for (_0, _1)
where
    _0: Validate,
    _1: Validate,
{
    type Valid = (_0::Valid, _1::Valid);
    fn validate(&self) -> Result<Self::Valid, String> {
        ({ self.0.validate()? }, { self.1.validate()? })
    }
}
impl<_0, _1, _2> Validate for (_0, _1, _2)
where
    _0: Validate,
    _1: Validate,
    _2: Validate,
{
    type Valid = (_0::Valid, _1::Valid, _2::Valid);
    fn validate(&self) -> Result<Self::Valid, String> {
        ({ self.0.validate()? }, { self.1.validate()? }, {
            self.2.validate()?
        })
    }
}
impl<_0, _1, _2, _3> Validate for (_0, _1, _2, _3)
where
    _0: Validate,
    _1: Validate,
    _2: Validate,
    _3: Validate,
{
    type Valid = (_0::Valid, _1::Valid, _2::Valid, _3::Valid);
    fn validate(&self) -> Result<Self::Valid, String> {
        (
            { self.0.validate()? },
            { self.1.validate()? },
            { self.2.validate()? },
            { self.3.validate()? },
        )
    }
}
