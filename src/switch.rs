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
use crate::systems::{self, EditorCfg,};

pub struct Cfg {
    target: NodePath,
    method: GodotString,
}

impl Cfg {
    const TARGET: &'static str = "unknown";
    const METHOD: &'static str = "unknown";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            target: NodePath::from_str(Self::TARGET),
            method: Self::METHOD.into(),
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
            name: "target",
            default: NodePath::from_str(Self::TARGET),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).target.new_ref(),
            setter: move |this: &mut T, targ| get_mut(this).target = targ,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "method",
            default: Self::METHOD.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).method.new_ref(),
            setter: move |this: &mut T, method| get_mut(this).method = method,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Default)]
pub struct Switch {
    cfg: Cfg,
}

impl godot::NativeClass for Switch {
    type Base = StaticBody2D;
    type UserData = MutexData<Switch>;

    fn class_name() -> &'static str {
        "Switch"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        Cfg::register_properties(builder, |this| &this.cfg, |this| &mut this.cfg);
    }
}

#[methods]
impl Switch {
    fn _init(_owner: StaticBody2D) -> Self {
        Default::default()
    }

    #[export]
    fn instance_init(&mut self, _owner: StaticBody2D, target: NodePath, method: GodotString) {
        self.cfg.target = target;
        self.cfg.method = method;
    }

    #[export]
    fn _ready(&self, owner: StaticBody2D) {
        unsafe { owner.to_node().add_to_group("switch".into(), false) };
        log::info!("Hello from switch.")
    }

    #[export]
    fn switch(&self, owner: StaticBody2D) {
        log::info!("Switch {:?} was hit!", unsafe { owner.get_name() }.to_string());
        unsafe {
            owner
                .get_node(self.cfg.target.new_ref())
                .map(|mut node| node.call(self.cfg.method.new_ref(), &[]));
        }
    }

    #[export]
    fn damage(&self, owner: StaticBody2D, _dmg: f64) {
        self.switch(owner);
    }
}

impl Switch {
    pub fn call_instance_init(mut switch: StaticBody2D, target: NodePath, method: GodotString) {
        let instance_init_method: GodotString = "instance_init".into();
        if unsafe { switch.has_method(instance_init_method.new_ref()) } {
            // TODO random aim based on how long was aimed for.
            unsafe {
                switch.call(instance_init_method, &[
                    Variant::from_node_path(&target),
                    Variant::from_godot_string(&method),
                ])
            };
        }
    }
    pub fn call_switch(mut switch: StaticBody2D) {
        let switch_method: GodotString = "switch".into();
        if unsafe { switch.has_method(switch_method.new_ref()) } {
            // TODO random aim based on how long was aimed for.
            unsafe {
                switch.call(switch_method, &[])
            };
        }
    }
}
