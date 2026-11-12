impl Validator<()> for () {
    type Error = ();
    fn validate(&self, value: &()) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = ();
        if valid { Ok(()) } else { Err(error) }
    }
}
impl<_0, _T0> Validator<(_T0,)> for (_0,)
where
    _0: Validator<_T0>, {
    type Error = (Option<_0::Error>,);
    fn validate(&self, value: &(_T0,)) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = ({
            let error = self.0.validate(&value.0).err();
            if error.is_some() {
                valid = false;
            }
        },);
        if valid { Ok(()) } else { Err(error) }
    }
}
impl<_0, _1, _T0, _T1> Validator<(_T0, _T1)> for (_0, _1)
where
    _0: Validator<_T0>,
    _1: Validator<_T1>, {
    type Error = (Option<_0::Error>, Option<_1::Error>);
    fn validate(&self, value: &(_T0, _T1)) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = (
            {
                let error = self.0.validate(&value.0).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.1.validate(&value.1).err();
                if error.is_some() {
                    valid = false;
                }
            },
        );
        if valid { Ok(()) } else { Err(error) }
    }
}
impl<_0, _1, _2, _T0, _T1, _T2> Validator<(_T0, _T1, _T2)> for (_0, _1, _2)
where
    _0: Validator<_T0>,
    _1: Validator<_T1>,
    _2: Validator<_T2>, {
    type Error = (Option<_0::Error>, Option<_1::Error>, Option<_2::Error>);
    fn validate(&self, value: &(_T0, _T1, _T2)) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = (
            {
                let error = self.0.validate(&value.0).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.1.validate(&value.1).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.2.validate(&value.2).err();
                if error.is_some() {
                    valid = false;
                }
            },
        );
        if valid { Ok(()) } else { Err(error) }
    }
}
impl<_0, _1, _2, _3, _T0, _T1, _T2, _T3> Validator<(_T0, _T1, _T2, _T3)> for (_0, _1, _2, _3)
where
    _0: Validator<_T0>,
    _1: Validator<_T1>,
    _2: Validator<_T2>,
    _3: Validator<_T3>,
{
    type Error = (
        Option<_0::Error>,
        Option<_1::Error>,
        Option<_2::Error>,
        Option<_3::Error>,
    );
    fn validate(&self, value: &(_T0, _T1, _T2, _T3)) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = (
            {
                let error = self.0.validate(&value.0).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.1.validate(&value.1).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.2.validate(&value.2).err();
                if error.is_some() {
                    valid = false;
                }
            },
            {
                let error = self.3.validate(&value.3).err();
                if error.is_some() {
                    valid = false;
                }
            },
        );
        if valid { Ok(()) } else { Err(error) }
    }
}
