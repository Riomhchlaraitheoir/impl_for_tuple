impl Notify for () {
    fn notify(&self) {
        ()
    }
}

impl<_0> Notify for (_0,)
where
    _0: Notify,
{
    fn notify(&self) {
        ({
            self.0.notify();
        },)
    }
}
impl<_0, _1> Notify for (_0, _1)
where
    _0: Notify,
    _1: Notify,
{
    fn notify(&self) {
        (
            {
                self.0.notify();
            },
            {
                self.1.notify();
            },
        )
    }
}
impl<_0, _1, _2> Notify for (_0, _1, _2)
where
    _0: Notify,
    _1: Notify,
    _2: Notify,
{
    fn notify(&self) {
        (
            {
                self.0.notify();
            },
            {
                self.1.notify();
            },
            {
                self.2.notify();
            },
        )
    }
}
impl<_0, _1, _2, _3> Notify for (_0, _1, _2, _3)
where
    _0: Notify,
    _1: Notify,
    _2: Notify,
    _3: Notify,
{
    fn notify(&self) {
        (
            {
                self.0.notify();
            },
            {
                self.1.notify();
            },
            {
                self.2.notify();
            },
            {
                self.3.notify();
            },
        )
    }
}
