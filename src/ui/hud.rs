use gdnative::{
    init::{Property, PropertyHint, PropertyUsage},
    Label,
    NodePath,
    Range,
    VBoxContainer,
};

#[derive(Debug)]
struct Cfg {
    arena_path: NodePath,
    hp_bar_path: NodePath,
    wave_display_path: NodePath,
}

impl Cfg {
    const ARENA_PATH: &'static str = "Arena";
    const HP_BAR_PATH: &'static str = "HPBar";
    const WAVE_DISPLAY_PATH: &'static str = "WaveNum";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            arena_path: NodePath::from_str(Self::ARENA_PATH),
            hp_bar_path: NodePath::from_str(Self::HP_BAR_PATH),
            wave_display_path: NodePath::from_str(Self::WAVE_DISPLAY_PATH),
        }
    }
}

#[derive(Debug)]
struct Data {
    max_hp: f64,
    hp: f64,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            max_hp: 100.,
            hp: 100.,
        }
    }
}

#[derive(Default)]
pub struct HUD {
    cfg: Cfg,
    data: Data,
}

impl godot::NativeClass for HUD {
    type Base = VBoxContainer;
    type UserData = godot::user_data::MutexData<HUD>;

    fn class_name() -> &'static str {
        "HUD"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        let default_usage = PropertyUsage::SCRIPT_VARIABLE | PropertyUsage::STORAGE | PropertyUsage::EDITOR;
        builder.add_property(Property {
            name: "arena",
            default: NodePath::from_str(Cfg::ARENA_PATH),
            hint: PropertyHint::None,
            getter: |this: &HUD| this.cfg.arena_path.new_ref(),
            setter: |this: &mut HUD, world| this.cfg.arena_path = world,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "hp_bar",
            default: NodePath::from_str(Cfg::HP_BAR_PATH),
            hint: PropertyHint::None,
            getter: |this: &HUD| this.cfg.hp_bar_path.new_ref(),
            setter: |this: &mut HUD, world| this.cfg.hp_bar_path = world,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "wave_display",
            default: NodePath::from_str(Cfg::WAVE_DISPLAY_PATH),
            hint: PropertyHint::None,
            getter: |this: &HUD| this.cfg.wave_display_path.new_ref(),
            setter: |this: &mut HUD, world| this.cfg.wave_display_path = world,
            usage: default_usage,
        });
    }
}

#[methods]
impl HUD {
    fn _init(_owner: VBoxContainer) -> Self {
        Self::default()
    }

    #[export]
    fn _ready(&mut self, _owner: VBoxContainer) {
        log::info!("Hello from the hud!");
    }

    #[export]
    fn set_max_health(&mut self, owner: VBoxContainer, max_hp: u64) {
        self.data.max_hp = f64::from_bits(max_hp);
        log::info!("Received max hp data ({}).", self.data.max_hp);
        if let Some(mut hp_bar) = unsafe {
            owner.get_node(self.cfg.hp_bar_path.new_ref()).and_then(|n| n.cast::<Range>())
        } {
            unsafe { hp_bar.set_max(self.data.max_hp); }
        }
    }

    #[export]
    fn set_health(&mut self, owner: VBoxContainer, hp: u64) {
        self.data.hp = f64::from_bits(hp);
        log::info!("Received hp data ({}).", self.data.hp);
        if let Some(mut hp_bar) = unsafe {
            owner.get_node(self.cfg.hp_bar_path.new_ref()).and_then(|n| n.cast::<Range>())
        } {
            unsafe { hp_bar.set_value(self.data.hp); }
        }
    }

    #[export]
    fn set_wave_num(&mut self, owner: VBoxContainer, wave_num: u64) {
        if let Some(mut wave_display) = unsafe {
            owner.get_node(self.cfg.wave_display_path.new_ref()).and_then(|n| n.cast::<Label>())
        } {
            unsafe { wave_display.set_text(format!("{:0>3}", wave_num).into()); }
        }
    }
}
