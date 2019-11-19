use serde::{Serialize, Deserialize};
use gdnative::{
    GodotString,
    NativeClass,
    Node,
    init::{ClassBuilder},
    user_data::MutexData,
    Variant,
};
use crate::{
    health::{System as HealthSys, Cfg as HealthCfg},
    systems::{self, EditorCfg},
};

#[derive(Debug)]
struct Cfg {
    dmg: f64,
}

impl Cfg {
    const DMG: f64 = 10.;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            dmg: Self::DMG,
        }
    }
}

#[derive(Default, Debug)]
pub struct Attack {
    cfg: Cfg,
}

impl NativeClass for Attack {
    type Base = Node;
    type UserData = MutexData<Attack>;

    fn class_name() -> &'static str {
        "Attack"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
    }
}

#[methods]
impl Attack {
    fn _init(mut owner: Node) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&self, _owner: Node) {
        log::info!("Hello from attacks!")
    }

    #[export]
    fn damage(&mut self, mut owner: Node, dmg: f64) {
    }
}

impl Attack {
    fn call_set_dmg(mut owner: Node, dmg: f64) {
        let set_dmg_method: GodotString = "set_dmg".into();
        if unsafe { owner.has_method(set_dmg_method.new_ref()) } {
            // TODO random aim based on how long was aimed for.
            unsafe {
                owner.call(set_dmg_method, &[Variant::from_f64(dmg)])
            };
        }
    }
}
