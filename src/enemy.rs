#[derive(godot::NativeClass)]
#[inherit(godot::Node)]
pub struct Simple;

#[gdnative::methods]
impl Simple {
    fn _init(_owner: gdnative::Node) -> Self {
        Simple
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("An enemy has appeared!")
    }
}
