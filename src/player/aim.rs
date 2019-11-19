use nalgebra as na;
use gdnative::{
    GodotString,
    NativeClass,
    Node,
    PackedScene,
    init::{ClassBuilder, Property, PropertyHint,},
    ResourceLoader,
    user_data::MutexData,
    NodePath,
};
use tap::TapOptionOps;
use std::{sync::{Arc, Mutex}, time::Duration};
use crate::{
    direction::Direction,
    systems::{self, EditorCfg},
    projectile::Normal as NormalProjectile,
};

pub struct Cfg {
    max_aim_time: Duration,
    min_aim_time: Duration,
    walk_speed: f64,
    normal_projectile: GodotString,
    world: NodePath,
    dmg: f64,
}

impl Cfg {
    const MAX_AIM: Duration = Duration::from_millis(300);
    const MIN_AIM: Duration = Duration::from_millis(100);
    const WALK_SPEED: f64 = 20.;
    const NORMAL_PROJECTILE_PATH: &'static str = "res://projectile.tscn";
    const WORLD: &'static str = "..";
    const DMG: f64 = 10.;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            max_aim_time: Self::MAX_AIM,
            min_aim_time: Self::MIN_AIM,
            walk_speed: Self::WALK_SPEED,
            normal_projectile: Self::NORMAL_PROJECTILE_PATH.into(),
            world: NodePath::from_str(Self::WORLD),
            dmg: Self::DMG,
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
            name: "aim/aim_from_nothing",
            default: Self::MAX_AIM.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).max_aim_time.as_millis() as u64,
            setter: move |this: &mut T, duration| get_mut(this).max_aim_time = Duration::from_millis(duration),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/aim_again",
            default: Self::MIN_AIM.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).min_aim_time.as_millis() as u64,
            setter: move |this: &mut T, cooldown| get_mut(this).min_aim_time = Duration::from_millis(cooldown as u64),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/projectile_scene",
            default: Self::NORMAL_PROJECTILE_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).normal_projectile.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).normal_projectile = path,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/walk_speed",
            default: Self::WALK_SPEED,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).walk_speed,
            setter: move |this: &mut T, walk_speed| get_mut(this).walk_speed = walk_speed,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/world",
            default: NodePath::from_str(Self::WORLD),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).world.new_ref(),
            setter: move |this: &mut T, world| get_mut(this).world = world,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Stage {
    WarmUp,
    Cooldown,
}

impl Stage {
    fn is_warm_up(&self) -> bool {
        if let Self::WarmUp = self {
            true
        } else {
            false
        }
    }
    fn is_cooldown(&self) -> bool {
        if let Self::Cooldown = self {
            true
        } else {
            false
        }
    }
}

struct Cache {
    projectile_scene: Arc<Mutex<PackedScene>>,
}

impl Cache {
    fn load_with(cfg: &Cfg) -> Option<Self> {
        let mut loader = ResourceLoader::godot_singleton();
        let loaded = loader.load(cfg.normal_projectile.new_ref(), "PackedScene".into(), false);
        loaded.and_then(|loaded| loaded.cast::<PackedScene>()).map(|scene| Self {
            projectile_scene: Arc::new(Mutex::new(scene)),
        })
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

pub struct Data {
    stage: Stage,
    pos: na::Vector2<f64>,
    time_to_aim: Duration,
}

impl Data {
    fn with_aim(cfg: &Cfg, pos: na::Vector2<f64>) -> Self {
        Self {
            pos,
            stage: Stage::WarmUp,
            time_to_aim: cfg.max_aim_time,
        }
    }
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

    pub fn is_aiming(&self) -> bool {
        self.data
            .as_ref()
            .map(|data| data.stage.is_warm_up())
            .unwrap_or(false)
    }

    pub fn aim_at(&mut self, pos: na::Vector2<f64>) {
        if let Some(data) = &mut self.data {
            data.pos = pos;
            if data.stage.is_cooldown() {
                // Hot start
                if data.time_to_aim < self.cfg.max_aim_time {
                    data.time_to_aim += self.cfg.min_aim_time;
                }
                data.stage = Stage::WarmUp;
            }
            // TODO handle how well aimed we are.
        } else {
            self.data = Some(Data::with_aim(&self.cfg, pos));
        }
    }

    pub fn narrow_aim(&mut self, delta: Duration) {
        if let Some(data) = &mut self.data {
            if data.time_to_aim != Duration::from_millis(0) {
                if data.time_to_aim < delta {
                    data.time_to_aim = Duration::from_millis(0);
                } else {
                    data.time_to_aim -= delta;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        // TODO disable any nodes that need to be disabled here.
        self.data = None;
    }

    pub fn calc_vel(&self, facing_dir: Direction) -> Option<na::Vector2<f64>> {
        if self.is_aiming() {
            Some(facing_dir.to_na_vec() * self.cfg.walk_speed)
        } else {
            None
        }
    }

    pub fn calc_dmg(&self) -> f64 {
        self.cfg.dmg
    }

    pub fn attack(&mut self, from: na::Vector2<f64>, owner: Node, dmg: f64) {
        (|| {
            // Unpack data & cache.
            let data = self.data.as_mut()
                .tap_none(|| log::warn!("Attempted to attack when not aiming... how?"))?;
            let cache = self.cache.as_ref()
                .tap_none(|| log::warn!("Failed to load cache earlier."))?;
            // Modify stage
            data.stage = Stage::Cooldown;
            // Init projectile
            let projectile = cache.projectile_scene
                .lock().ok().tap_none(|| log::warn!("Failed to lock loaded projectile scene."))?
                .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                .tap_none(|| log::warn!("Could not instance projectile scene."))?;
            let direction = (data.pos - from).normalize();
            NormalProjectile::call_init_instance(
                unsafe { projectile.cast() }
                    .tap_none(|| log::warn!("Projectile is not a KinematicBody2D, which is unexpected."))?,
                from + 20. * direction,
                direction,
                dmg,
            );
            // Add to tree, and let it make its own way in the world.
            unsafe {
                owner
                    .get_node(self.cfg.world.new_ref())
                    .tap_none(|| log::warn!("Provided world to `Aim` system is incorrect."))?
                    .add_child(Some(projectile), false)
            }
            Some(())
        })();
    }
}
