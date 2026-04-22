use dioxus::prelude::*;

fn check_signal_set() {
    let s = Signal::new(0);
    s.set(1); // If this compiles, it takes &self
}

fn check_signal_set_mut() {
    let mut s = Signal::new(0);
    s.set(1); // If this only compiles with mut, it takes &mut self
}
