use bevy::{prelude::*, window::close_on_esc};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(close_on_esc)
        .add_system(print_hi)
        .run();
}

fn print_hi() {
    println!("Hi");
}
