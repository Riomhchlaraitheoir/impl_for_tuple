impl Notify for Tuple<{ ..=4 }> where Self::Element<{..}>: Notify {
    fn notify(&self) {
        #[unroll_loop]
        for i in 0..Self::Length {
            self.[i].notify();
        }
    }
}