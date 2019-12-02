use gdnative::{
    init::{Property, PropertyHint, PropertyUsage, Signal,},
    NodePath,
    Control,
};

mod hud;
pub use hud::HUD;
mod end;
pub use end::End;
mod start;
pub use start::Start;

#[derive(Debug)]
struct Cfg {
    hud_path: NodePath,
}

impl Cfg {
    const HUD_PATH: &'static str = "HUD";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            hud_path: NodePath::from_str(Self::HUD_PATH),
        }
    }
}

#[derive(Default)]
struct UI {
    cfg: Cfg,
}

impl UI {
    const BEGIN_WAVE: &'static str = "trigger_wave";
}

impl godot::NativeClass for UI {
    type Base = Control;
    type UserData = godot::user_data::MutexData<UI>;

    fn class_name() -> &'static str {
        "UI"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        let default_usage = PropertyUsage::SCRIPT_VARIABLE | PropertyUsage::STORAGE | PropertyUsage::EDITOR;
        builder.add_property(Property {
            name: "hud",
            default: NodePath::from_str(Cfg::HUD_PATH),
            hint: PropertyHint::None,
            getter: |this: &UI| this.cfg.hud_path.new_ref(),
            setter: |this: &mut UI, world| this.cfg.hud_path = world,
            usage: default_usage,
        });
    }
}

#[methods]
impl UI {
    fn _init(_owner: Control) -> Self {
        Self::default()
    }

    #[export]
    fn _ready(&mut self, _owner: Control) {
        log::info!("Hello from the hud!");
    }

    #[export]
    fn _input(&self, owner: Control) {
        // TODO Some sort of keyboard input.
    }
}
