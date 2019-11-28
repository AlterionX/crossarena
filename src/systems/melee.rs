use nalgebra as na;
use gdnative::{
    NativeClass,
    Node,
    init::{ClassBuilder, Property, PropertyHint,},
    Instance,
    user_data::MutexData,
    NodePath,
};
use std::time::Duration;
use tap::{TapResultOps, TapOptionOps};
use crate::{
    entity::MeleeAttack,
    util::Direction,
    systems::{self, System as SysTrait, EditorCfg},
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

pub struct Cache {}

impl Cache {
    fn load_with(_: &Cfg) -> Option<Self> {
        Some(Self {})
    }

    fn attack(&self, attack_id: u64, cfg: &Cfg, owner: Node) -> Option<Instance<MeleeAttack>> {
        Instance::try_from_base(unsafe {
            owner
                .get_node(cfg.frame_nodes_path.new_ref())?
                .get_node(attack_id.to_string().into())?
                .cast()?
        })
    }
}

struct AttackData {
    next_attack: Option<u64>,
    cooldown: Duration,
}

impl AttackData {
    fn from_attack(atk: &MeleeAttack) -> Self {
        Self {
            next_attack: atk.next_attack(),
            cooldown: atk.cooldown(),
        }
    }
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
    pub fn reset(&mut self, owner: Node) {
        let data = self.data.take();
        let cache = match self.cache.as_ref() {
            Some(cache) => cache,
            None => {
                log::warn!(
                    "Could not find melee cache when attempting to reset melee systems of {}.",
                    unsafe { owner.get_name() }.to_string(),
                );
                return;
            }
        };
        let cfg = &self.cfg;

        if let Some(data) = data {
            let atk = match cache.attack(data.curr_attack, cfg, owner) {
                Some(atk) => atk,
                None => {
                    log::warn!(
                        "Could not find current attack {} in node {}. How did that happen?",
                        unsafe { owner.get_name() }.to_string(),
                        data.curr_attack,
                    );
                    return;
                }
            };
            match atk.map_mut(|atk, base| atk.reset(base)) {
                Ok(_) => (),
                Err(_) => log::warn!(
                    "Could not reset attack node when resetting melee system of {} node.!",
                    unsafe { owner.get_name() }.to_string(),
                )
            }
        }
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
    pub fn attack(&mut self, owner: Node, dir: Direction) {
        let atk_and_id = (|| if self.is_attacking() {
            let (cfg, cache, data) = self.res_view()
                .tap_err(|e| {
                    log::warn!(
                        "While node {} attempted to attack (melee) \
                        encountered the following error:\n\t{}",
                        unsafe { owner.get_name() }.to_string(),
                        e,
                    )
                }).ok()?;
            if data.since_last > data.attack.cooldown {
                None
            } else {
                data.attack.next_attack.and_then(|next_attack_id| Some((
                    next_attack_id,
                    cache.attack(next_attack_id, cfg, owner)?
                )))
            }
        } else {
            let cache = self.cache
                .as_ref()
                .tap_none(|| {
                    log::warn!(
                        "While node {} attempted to attack (melee), could not locate cache.",
                        unsafe { owner.get_name() }.to_string(),
                    )
                })?;
            Some((self.cfg.initial_attack, cache.attack(self.cfg.initial_attack, &self.cfg, owner)?))
        })();
        let data = atk_and_id.and_then(|(id, atk)| {
            self.reset(owner);
            atk
                .map_mut(|atk, base| {
                    atk.execute(base, dir);
                    Data {
                        attack: AttackData::from_attack(atk),
                        since_last: Duration::from_millis(0),
                        curr_attack: id,
                    }
                })
                .tap_err(|_| log::warn!(
                    "Failed to excute attack in {} node.",
                    unsafe { owner.get_name() }.to_string(),
                ))
                .ok()
        });
        if let Some(data) = data {
            self.data = Some(data);
        }
    }
}

impl SysTrait for System {
    type Cfg = Cfg;
    type Cache = Cache;
    type Data = Data;

    fn view(&self) -> (&Self::Cfg, Option<&Self::Cache>, Option<&Self::Data>) {
        (&self.cfg, self.cache.as_ref(), self.data.as_ref())
    }
    fn view_mut(&mut self) -> (&mut Self::Cfg, Option<&mut Self::Cache>, Option<&mut Self::Data>) {
        (&mut self.cfg, self.cache.as_mut(), self.data.as_mut())
    }
}
