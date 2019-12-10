use gdnative::{
    self as godot,
    GodotString,
    init::{Property, PropertyHint, ClassBuilder,},
    StaticBody2D,
    NativeClass,
    NodePath,
    user_data::MutexData,
    Variant,
};
use crate::{util::Group, systems::{self, EditorCfg,}};

pub struct Cfg {
    crafting_ui: NodePath,
}

impl Cfg {
    const CRAFTING_UI: &'static str = "unknown";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            crafting_ui: NodePath::from_str(Self::CRAFTING_UI),
        }
    }
}

impl EditorCfg for Cfg {
    fn register_properties<T, G, GM>(
        builder: &ClassBuilder<T>,
        get_proto: G,
        get_mut_proto: GM,
    )
        where
            T: Send + NativeClass<UserData = MutexData<T>>,
            G: Clone + Fn(&T) -> &Self,
            GM: Clone + Fn(&mut T) -> &mut Self,
    {
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "crafting_ui",
            default: NodePath::from_str(Self::CRAFTING_UI),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).crafting_ui.new_ref(),
            setter: move |this: &mut T, targ| get_mut(this).crafting_ui = targ,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Default)]
pub struct Forge {
    cfg: Cfg,
}

impl godot::NativeClass for Forge {
    type Base = StaticBody2D;
    type UserData = MutexData<Forge>;

    fn class_name() -> &'static str {
        "Forge"
    }

    fn init(_owner: Self::Base) -> Self {
        Default::default()
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        Cfg::register_properties(builder, |this| &this.cfg, |this| &mut this.cfg);
    }
}

#[methods]
impl Forge {
    #[export]
    fn instance_init(&mut self, _owner: StaticBody2D, crafting_ui: NodePath) {
        self.cfg.crafting_ui = crafting_ui;
    }

    #[export]
    fn _ready(&self, owner: StaticBody2D) {
        Group::Switch.add_node(unsafe { owner.to_node() });
        log::info!("Hello from switch.")
    }

    #[export]
    fn switch(&self, owner: StaticBody2D) {
        log::info!("Forge {:?} was hit!", unsafe { owner.get_name() }.to_string());
        // TODO pause game and open crafting menu.
    }

    #[export]
    fn damage(&self, owner: StaticBody2D, _dmg: f64) {
        self.switch(owner);
    }
}

impl Forge {
    pub fn call_instance_init(mut switch: StaticBody2D, crafting_ui: NodePath) {
        let instance_init_method: GodotString = "instance_init".into();
        if unsafe { switch.has_method(instance_init_method.new_ref()) } {
            // TODO random aim based on how long was aimed for.
            unsafe {
                switch.call(instance_init_method, &[
                    Variant::from_node_path(&crafting_ui),
                ])
            };
        }
    }
}
