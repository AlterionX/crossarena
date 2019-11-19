use nalgebra as na;
use gdnative::{
    NativeClass,
    Node,
    Node2D,
    init::{ClassBuilder, Property, PropertyHint,},
    user_data::MutexData,
    NodePath,
};
use std::time::Duration;
use crate::{
    conv,
    direction::Direction,
    systems::{self, EditorCfg},
};

pub struct Cfg {
    pub frame_nodes_path: NodePath,
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

    fn attack(&self, attack_id: u64, cfg: &Cfg, owner: Node) -> Option<Attack> {
        Some(Attack(unsafe {
            owner
                .get_node(cfg.frame_nodes_path.new_ref())?
                .get_node(attack_id.to_string().into())?
        }))
    }
}

// TODO Implement a script for this for an actual node.
struct Attack(Node);

impl Attack {
    fn next_attack(&mut self) -> Option<u64> {
        // Next attack
        let next_atk_fn = "next_attack".into();
        unsafe { self.0.call(next_atk_fn, &[]).try_to_u64() }
    }
    fn cooldown(&mut self) -> Option<Duration> {
        // Cooldown until can use another attack.
        let cooldown_fn = "cooldown".into();
        let cooldown = unsafe {
            self.0.call(cooldown_fn, &[])
        }.try_to_u64()?;
        Some(Duration::from_millis(cooldown))
    }
    fn data(&mut self) -> Option<AttackData> {
        // Gather everything at last.
        Some(AttackData {
            next_attack: self.next_attack()?,
            cooldown: self.cooldown()?,
        })
    }
    fn execute(&mut self, from: na::Vector2<f64>, dir: Direction) {
        // TODO Marshal args.
        unsafe {
            self.0.call("execute".into(), &[
                (&conv::na64_to_g(from)).into(),
                (&conv::na64_to_g(dir.to_na_vec())).into(),
            ]);
        }
        // node.set_transform(_);
        // node.set_disabled(false);
        // node.call("apply_damage", &[]);
        // node.set_disabled(true);
    }
}

struct AttackData {
    next_attack: u64,
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
    pub fn attack(&mut self, owner: Node2D, dir: Direction) {
        let own_pos = conv::g_to_na64(unsafe { owner.get_global_position() });
        let owner = unsafe { owner.to_node() };
        if self.data.is_some() {
            (|| {
                let data = self.data.as_mut()?;
                let cache = self.cache.as_ref()?;
                let mut attack = cache.attack(data.curr_attack, &self.cfg, owner)?;
                let attack_data = attack.data()?;
                // TODO Grab data from attack nodes.
                if data.since_last > attack_data.cooldown {
                    // Do nothing
                } else {
                    let mut next_attack = cache.attack(attack_data.next_attack, &self.cfg, owner)?;
                    self.data = Some(Data {
                        attack: next_attack.data()?,
                        since_last: Duration::from_millis(0),
                        curr_attack: attack_data.next_attack,
                    });
                    next_attack.execute(own_pos, dir);
                }
                Some(())
            })();
        } else {
            (|| {
                let cache = self.cache.as_ref()?;
                let mut initial_attack = cache.attack(self.cfg.initial_attack, &self.cfg, owner)?;
                // Create attack.
                self.data = Some(Data {
                    attack: initial_attack.data()?,
                    since_last: Duration::from_millis(0),
                    curr_attack: self.cfg.initial_attack,
                });
                initial_attack.execute(own_pos, dir);
                Some(())
            })();
        }
    }
}
