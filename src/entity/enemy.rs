use gdnative::{
    NativeClass,
    Node,
    NodePath,
    KinematicBody2D,
    init::{ClassBuilder},
    FromVariant,
    ToVariant,
    user_data::MutexData,
};
use crate::{
    util::Group,
    systems::{
        EditorCfg,
        health::{System as HealthSys, Cfg as HealthCfg},
    },
};

#[derive(Debug, Clone)]
#[derive(ToVariant, FromVariant)]
pub struct Cfg {
    pub id: u64,
    pub value: f64,
    pub health: u64,
    pub available_from: u64,
    pub blacklist: Vec<u64>,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            id: 0,
            value: 1.,
            health: 100,
            available_from: 0,
            blacklist: vec![],
        }
    }
}

impl Cfg {
    pub fn call_get_cfg(mut owner: Node) -> Option<Cfg> {
        Self::from_variant(unsafe { &owner.call("get_cfg".into(), &[]) })
    }
    pub fn call_set_target(mut owner: Node, target: NodePath) {
        unsafe {
            owner.call("set_target".into(), &[target.to_variant()]);
        }
    }
}

#[derive(Debug)]
pub struct Data {
    target: NodePath,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            target: ".".into(),
        }
    }
}

#[derive(Default, Debug)]
pub struct SimpleEnemy {
    cfg: Cfg,
    data: Option<Data>,

    health: HealthSys,
}

impl NativeClass for SimpleEnemy {
    type Base = KinematicBody2D;
    type UserData = MutexData<SimpleEnemy>;

    fn class_name() -> &'static str {
        "SimpleEnemy"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        HealthCfg::register_properties(builder, |this| &this.health.cfg, |this| &mut this.health.cfg);
    }
}

#[methods]
impl SimpleEnemy {
    fn _init(owner: KinematicBody2D) -> Self {
        Group::Enemy.add_node(unsafe { owner.to_node() });
        Default::default()
    }

    #[export]
    fn _ready(&mut self, _owner: KinematicBody2D) {
        self.health.init();
        self.cfg.health = self.health.get_max_hp();
        log::info!("An enemy has spawned!")
    }

    #[export]
    fn _exit_tree(&mut self, _owner: KinematicBody2D) {
        log::info!("SimpleEnemy to be deleted.");
    }

    #[export]
    fn _physics_process(&self, _owner: KinematicBody2D, _delta: f64) {
    }

    #[export]
    fn damage(&mut self, mut owner: KinematicBody2D, dmg: f64) {
        self.health.damage(dmg, None);
        if self.health.is_dead() {
            // TODO Any other cleanup.
            unsafe { owner.queue_free() };
        }
    }

    #[export]
    fn set_target(&mut self, _: KinematicBody2D, target: NodePath) {
        self.data.as_mut().map(|data| {
            data.target = target;
            data
        });
    }

    #[export]
    fn get_cfg(&mut self, _: KinematicBody2D) -> Cfg {
        self.cfg.clone()
    }
}
