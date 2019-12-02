use gdnative::{NativeClass, Control};

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Start;

#[methods]
impl Start {
    fn _init(owner: Control) -> Self {
        Self
    }
}
