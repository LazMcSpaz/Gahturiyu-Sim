//! The playtest window (Bevy). It only reads the simulation and passes on
//! orders; nothing about how the world behaves lives here.

mod view;

fn main() {
    view::app::run();
}
