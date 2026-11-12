use std::cell::Cell;
use impl_for_tuple::impl_for_tuple;

trait Notify {
    fn notify(&self);
}

impl_for_tuple! {
    impl Notify for Tuple where Self::Element<{..}>: Notify {
        fn notify(&self) {
            #[unroll_loop]
            for i in 0..Self::Length {
                self.[i].notify();
            };
        }
    }
}

impl Notify for Cell<bool> {
    fn notify(&self) {
        self.set(true);
    }
}

fn main() {
    let tuple = (
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
        Cell::new(false),
    );
    tuple.notify();
    assert!(tuple.0.get());
    assert!(tuple.1.get());
    assert!(tuple.2.get());
    assert!(tuple.3.get());
    assert!(tuple.4.get());
    assert!(tuple.5.get());
    assert!(tuple.6.get());
    assert!(tuple.7.get());
    assert!(tuple.8.get());
}