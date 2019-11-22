use serde::{Serialize, Deserialize};
use gdnative::{
    NativeClass,
    KinematicBody2D,
    init::{ClassBuilder},
    user_data::MutexData,
};
use crate::{
    health::{System as HealthSys, Cfg as HealthCfg},
    systems::{self, EditorCfg},
};

pub const ENEMY: &'static str = "enemy";

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[derive(Debug, Clone)]
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
            health: 1000,
            available_from: 0,
            blacklist: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[derive(Debug, Clone, Copy)]
pub struct Data {
}

impl Default for Data {
    fn default() -> Self {
        Self {
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
    fn _init(mut owner: KinematicBody2D) -> Self {
        unsafe {
            owner.add_to_group(ENEMY.into(), false);
        }
        Default::default()
    }

    #[export]
    fn _ready(&mut self, _owner: KinematicBody2D) {
        self.health.init();
        log::info!("An enemy has spawned!")
    }

    #[export]
    fn _exit_tree(&mut self, _owner: KinematicBody2D) {
        log::info!("SimpleEnemy to be deleted.");
    }

    #[export]
    fn damage(&mut self, mut owner: KinematicBody2D, dmg: f64) {
        self.health.damage(dmg, None);
        if self.health.is_dead() {
            // TODO Any other cleanup.
            unsafe { owner.queue_free() };
        }
    }
}
