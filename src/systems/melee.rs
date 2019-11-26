use nalgebra as na;
use gdnative::{
    Area2D,
    NativeClass,
    Node,
    Node2D,
    init::{ClassBuilder, Property, PropertyHint,},
    user_data::MutexData,
    NodePath,
};
use std::time::Duration;
use tap::TapOptionOps;
use crate::{
    entity::MeleeAttack,
    util::Direction,
    systems::{self, EditorCfg},
};

pub struct Cfg {
    frame_nodes_path: NodePath,
    walk_speed: f64,
    initial_attack: u64,
}

impl Cfg {
    const FRAME_NODE_PATH: &'static str = "Melee";
    const WALK_SPEED: f64 = 0.;
    const INITIAL_ATTACK: u64 = 0;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            frame_nodes_path: NodePath::from_str(Self::FRAME_NODE_PATH),
            walk_speed: Self::WALK_SPEED,
            initial_attack: Self::INITIAL_ATTACK,
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
            name: "melee/frames",
            default: NodePath::from_str(Self::FRAME_NODE_PATH),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).frame_nodes_path.new_ref(),
            setter: move |this: &mut T, path| get_mut(this).frame_nodes_path = path,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

struct Cache {
    melee_combos: u64,
}

impl Cache {
    fn load_with(cfg: &Cfg) -> Option<Self> {
        Some(Self {
            melee_combos: 0,
        })
    }

    fn attack(&self, attack_id: u64, cfg: &Cfg, owner: Node) -> Option<AttackRef> {
        Some(AttackRef(unsafe {
            owner
                .get_node(cfg.frame_nodes_path.new_ref())?
                .get_node(attack_id.to_string().into())?
                .cast()?
        }))
    }
}

// TODO Implement a script for this for an actual node.
struct AttackRef(Area2D);

impl AttackRef {
    fn next_attack(&mut self) -> Option<u64> {
        MeleeAttack::call_next_attack(&mut self.0)
    }
    fn cooldown(&mut self) -> Option<Duration> {
        MeleeAttack::call_cooldown(&mut self.0)
    }
    fn data(&mut self) -> Option<AttackData> {
        // Gather everything at last.
        Some(AttackData {
            next_attack: self.next_attack(),
            cooldown: self.cooldown()?,
        })
    }
    fn execute(&mut self, dir: Direction) {
        MeleeAttack::call_execute(self.0, dir)
    }
}

struct AttackData {
    next_attack: Option<u64>,
    cooldown: Duration,
}

pub struct Data {
    attack: AttackData,
    since_last: Duration,
    curr_attack: u64,
}

#[derive(Default)]
pub struct System {
    pub cfg: Cfg,
    cache: Option<Cache>,
    pub data: Option<Data>,
}

impl System {
    pub fn load_cache(&mut self) {
        self.cache = Cache::load_with(&self.cfg);
    }
    pub fn reset(&mut self) {
        self.data = None;
    }
    pub fn calc_vel(&self, facing_dir: Direction) -> Option<na::Vector2<f64>> {
        self.data.as_ref().map(|_| (facing_dir.to_na_vec() * self.cfg.walk_speed))
    }
    pub fn process(&mut self, delta: Duration) {
        if let Some(data) = self.data.as_mut() {
            data.since_last += delta;
            if data.since_last > data.attack.cooldown {
                self.data = None;
            }
        }
    }
    pub fn is_attacking(&self) -> bool {
        self.data.is_some()
    }
    pub fn attack(&mut self, owner: Node2D, dir: Direction) {
        let owner = unsafe { owner.to_node() };
        if self.is_attacking() {
            (|| {
                let frames_path = self.cfg.frame_nodes_path.new_ref();
                let data = self.data.as_mut()
                    .tap_none(|| log::warn!("Melee system is attacking but has no data."))?;
                let cache = self.cache.as_ref()
                    .tap_none(|| log::warn!("Cache loading failed earlier."))?;
                let mut attack = cache.attack(data.curr_attack, &self.cfg, owner)
                    .tap_none(|| log::warn!("Could not locate target attack {:?} in melee attack node: {}.", data.curr_attack, frames_path.new_ref().to_string()))?;
                let attack_data = attack.data()
                    .tap_none(|| log::warn!("Could not access attack data of {:?} attack.", data.curr_attack))?;
                // TODO Grab data from attack nodes.
                if data.since_last > attack_data.cooldown {
                    // Do nothing
                } else {
                    let next_attack_id = attack_data.next_attack.unwrap_or(self.cfg.initial_attack);
                    let mut next_attack = cache.attack(next_attack_id, &self.cfg, owner)?;
                    let attack_data = next_attack.data()
                        .tap_none(|| log::warn!("Could not access attack data of {:?} attack.", next_attack_id))?;
                    self.data = Some(Data {
                        attack: attack_data,
                        since_last: Duration::from_millis(0),
                        curr_attack: next_attack_id,
                    });
                    next_attack.execute(dir);
                }
                Some(())
            })();
        } else {
            (|| {
                let cache = self.cache.as_ref()
                    .tap_none(|| log::warn!("Cache loading failed earlier."))?;
                let mut initial_attack = cache.attack(self.cfg.initial_attack, &self.cfg, owner)?;
                let attack_data = initial_attack.data()
                    .tap_none(|| log::warn!("Could not access attack data of {:?} attack.", self.cfg.initial_attack))?;
                // Create attack.
                self.data = Some(Data {
                    attack: attack_data,
                    since_last: Duration::from_millis(0),
                    curr_attack: self.cfg.initial_attack,
                });
                initial_attack.execute(dir);
                Some(())
            })();
        }
    }
}
