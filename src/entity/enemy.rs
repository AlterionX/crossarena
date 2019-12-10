use nalgebra as na;
use gdnative::{
    NativeClass,
    Node,
    Node2D,
    NodePath,
    KinematicBody2D,
    init::{ClassBuilder},
    FromVariant,
    ToVariant,
    user_data::MutexData,
};
use crate::{
    util::{conv, Group},
    systems::{
        EditorCfg,
        health::{System as HealthSys, Cfg as HealthCfg},
        items,
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
    pub fn call_set_drop_table(mut owner: Node, drop_table: items::DropTable) {
        unsafe {
            owner.call("set_drops".into(), &[drop_table.to_variant()]);
        }
    }
}

#[derive(Default, Debug)]
pub struct Data {
}

#[derive(Default, Debug)]
pub struct State {
    target: Option<NodePath>,
    drop_table: Option<items::DropTable>,
}

#[derive(Default, Debug)]
pub struct SimpleEnemy {
    cfg: Cfg,
    state: State,
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

impl SimpleEnemy {
    const BASE_VELOCITY: f64 = 100.;
    const CONTACT_DAMAGE: f64 = 50.;

    fn get_target(&self, owner: &KinematicBody2D) -> Option<Node2D> {
        unsafe {
            owner.get_node(self.state.target.as_ref()?.new_ref())?.cast()
        }
    }

    fn calc_vel(&self) -> f64 {
        Self::BASE_VELOCITY
    }

    fn calc_dmg(&self) -> f64 {
        Self::CONTACT_DAMAGE
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
    }

    #[export]
    fn _exit_tree(&mut self, _owner: KinematicBody2D) {
        log::info!("SimpleEnemy to be deleted.");
    }

    #[export]
    fn _physics_process(&self, mut owner: KinematicBody2D, delta: f64) {
        let move_dir = if let Some(target) = self.get_target(&owner) {
            let targ_pos = conv::g_to_na64(unsafe { target.get_global_position() });
            let own_pos = conv::g_to_na64(unsafe { owner.get_global_position() });
            let dir = targ_pos - own_pos;
            let norm = dir.norm();
            dir / norm
        } else {
            na::Vector2::zeros()
        };

        let col = unsafe {
            owner.move_and_collide(
                conv::na64_to_g(move_dir * self.calc_vel() * delta),
                true,
                true,
                false,
            )
        };
        let col = col.and_then(|col| unsafe { col.get_collider()?.cast::<Node>() });
        if let Some(col) = col {
            if Group::Player.has_node(col) {
                HealthSys::call_damage(unsafe { col.to_object() }, self.calc_dmg());
            }
        }
    }

    #[export]
    fn damage(&mut self, mut owner: KinematicBody2D, dmg: f64) {
        log::info!("Damage applied!");
        self.health.damage(dmg, None);
        if self.health.is_dead() {
            // TODO Any other cleanup.
            unsafe { owner.queue_free() };
        }
    }

    #[export]
    fn set_target(&mut self, _: KinematicBody2D, target: NodePath) {
        self.state.target = Some(target);
    }

    #[export]
    fn get_cfg(&mut self, _: KinematicBody2D) -> Cfg {
        self.cfg.clone()
    }

    #[export]
    fn set_drops(&mut self, _: KinematicBody2D, table: items::DropTable) {
        self.state.drop_table = Some(table)
    }

    pub fn get_drops(&self, wave: u64) -> Vec<items::Stack> {
        self.state.drop_table
            .as_ref()
            .map(|table| table.generate_drops(wave))
            .unwrap_or(vec![])
    }
}
