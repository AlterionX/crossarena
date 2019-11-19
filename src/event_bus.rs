use gdnative::{
    self as godot,
    NativeClass,
    Node,
};

#[derive(Default, Debug)]
#[derive(NativeClass)]
#[inherit(Node)]
pub struct EventBus;

#[gdnative::methods]
impl EventBus {
    fn _init(_owner: Node) -> Self {
        Default::default()
    }

}
