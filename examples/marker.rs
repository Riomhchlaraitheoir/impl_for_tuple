use impl_for_tuple::impl_for_tuple;

trait Marker {}

impl_for_tuple! {
    impl Marker for Tuple where Self::Element<{..}>: Marker {}
}

fn main() {
    assert_marker((45_u8, 32_u16, "foo"))
}

fn assert_marker(_: impl Marker) {}

impl Marker for u8 {}
impl Marker for u16 {}
impl Marker for &'static str {}