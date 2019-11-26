use nalgebra as na;
use gdnative::{
    Control,
    GlobalConstants,
    GodotString,
    init::{ClassBuilder, Property, PropertyHint,},
    NativeClass,
    Node,
    Node2D,
    NodePath,
    PackedScene,
    ResourceLoader,
    user_data::MutexData,
};
use tap::TapOptionOps;
use rand::distributions::{Distribution, Uniform};
use std::{ops::RangeInclusive, sync::{Arc, Mutex}, time::Duration};
use crate::{
    util::{conv, Direction, common_matrices as common_mats},
    systems::{self, EditorCfg},
    entity::NormalProjectile,
};

#[derive(Debug, PartialEq)]
pub struct Cfg {
    max_aim_time: Duration,
    min_aim_time: Duration,
    cooldown_time: Duration,
    aim_range_off_rot: f64,
    walk_speed: f64,
    normal_projectile: GodotString,
    world: NodePath,
    upper_fan: NodePath,
    lower_fan: NodePath,
    fan_offset: f64,
    dmg: f64,
}

impl Cfg {
    const MIN_AIM: Duration = Duration::from_millis(100);
    const MAX_AIM: Duration = Duration::from_millis(3000);
    const COOLDOWN: Duration = Duration::from_millis(100);
    const AIM_OFF: f64 = std::f64::consts::FRAC_PI_2;
    const WALK_SPEED: f64 = 20.;
    const NORMAL_PROJECTILE_PATH: &'static str = "res://projectile.tscn";
    const WORLD: &'static str = "..";
    const UPPER_FAN: &'static str = "LowerFan";
    const LOWER_FAN: &'static str = "UpperFan";
    const FAN_OFFSET: f64 = 50.;
    const DMG: f64 = 10.;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            max_aim_time: Self::MAX_AIM,
            min_aim_time: Self::MIN_AIM,
            cooldown_time: Self::COOLDOWN,
            aim_range_off_rot: Self::AIM_OFF,
            walk_speed: Self::WALK_SPEED,
            normal_projectile: Self::NORMAL_PROJECTILE_PATH.into(),
            world: NodePath::from_str(Self::WORLD),
            lower_fan: NodePath::from_str(Self::UPPER_FAN),
            upper_fan: NodePath::from_str(Self::LOWER_FAN),
            fan_offset: Self::FAN_OFFSET,
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
            name: "aim/cooldown",
            default: Self::COOLDOWN.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).cooldown_time.as_millis() as u64,
            setter: move |this: &mut T, cooldown| get_mut(this).cooldown_time = Duration::from_millis(cooldown as u64),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/bad_aim_offset",
            default: Self::AIM_OFF,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).aim_range_off_rot,
            setter: move |this: &mut T, aim_off| get_mut(this).aim_range_off_rot = aim_off,
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
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/upper_fan",
            default: NodePath::from_str(Self::UPPER_FAN),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).upper_fan.new_ref(),
            setter: move |this: &mut T, upper_fan| get_mut(this).upper_fan = upper_fan,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/lower_fan",
            default: NodePath::from_str(Self::LOWER_FAN),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).lower_fan.new_ref(),
            setter: move |this: &mut T, lower_fan| get_mut(this).lower_fan = lower_fan,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/fan_offset",
            default: Self::FAN_OFFSET,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).fan_offset,
            setter: move |this: &mut T, fan_offset| get_mut(this).fan_offset = fan_offset,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/dmg",
            default: Self::DMG,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).dmg,
            setter: move |this: &mut T, dmg| get_mut(this).dmg = dmg,
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

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
pub struct Data {
    stage: Stage,
    pos: na::Vector2<f64>,
    time_to_aim: Duration,
    cooling_down: Duration,
}

impl Data {
    fn with_aim(cfg: &Cfg, pos: na::Vector2<f64>) -> Self {
        Self {
            pos,
            stage: Stage::WarmUp,
            time_to_aim: cfg.max_aim_time,
            cooling_down: Duration::from_millis(0),
        }
    }

    fn aim_at(&mut self, cfg: &Cfg, pos: na::Vector2<f64>) {
        self.pos = pos;
        if self.stage.is_cooldown() {
            // Hot start
            if self.time_to_aim < cfg.max_aim_time {
                self.time_to_aim += cfg.min_aim_time;
            }
            self.stage = Stage::WarmUp;
            self.cooling_down = Duration::from_millis(0);
        }
    }

    fn step_time(&mut self, cfg: &Cfg, delta: Duration) -> bool {
        match self.stage {
            Stage::WarmUp if self.time_to_aim != Duration::from_millis(0) => {
                if self.time_to_aim < delta {
                    self.time_to_aim = Duration::from_millis(0);
                } else {
                    self.time_to_aim -= delta;
                }
                false
            },
            Stage::Cooldown => {
                self.cooling_down += delta;
                self.cooling_down >= cfg.cooldown_time
            },
            _ => false,
        }
    }

    fn possible_angle_offsets(&self, cfg: &Cfg) -> RangeInclusive<f64> {
        let percentage_to_full_aim = self.time_to_aim.as_secs_f64() / cfg.max_aim_time.as_secs_f64();
        log::info!("Aim off from ideal: {}, {:?}, {:?}", percentage_to_full_aim * 100., self.time_to_aim, cfg.max_aim_time);
        let off_angle = cfg.aim_range_off_rot * percentage_to_full_aim;
        let off_angle = off_angle.abs();
        -off_angle..=off_angle
    }

    fn true_dir(&self, cfg: &Cfg, from: na::Vector2<f64>) -> na::Vector2<f64> {
        let (ideal, aim_duration) = {
            ((self.pos - from).normalize(), self.time_to_aim)
        };
        log::info!("Time and max: {:?}, {:?}", aim_duration, cfg.max_aim_time);
        if aim_duration > Duration::from_millis(0) {
            let aim_distribution = Uniform::from(self.possible_angle_offsets(cfg));
            let aim_off = aim_distribution.sample(&mut rand::thread_rng());
            let rot_mat = common_mats::rotation(aim_off);
            rot_mat * ideal
        } else {
            ideal
        }
    }
}

#[derive(Default, Debug)]
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

    fn align_single_fan(&self, fan: Node, pos: na::Vector2<f64>, dir: na::Vector2<f64>, rotation: f64) {
        if let Some(mut fan) = unsafe { fan.cast::<Control>() } {
            let offset = common_mats::rotation(rotation) * dir;
            let offset = offset.normalize() * self.cfg.fan_offset;
            unsafe {
                // this is in global coords
                fan.set_global_position(conv::na64_to_g(pos + offset));
                fan.set_size(gdnative::Vector2::new(dir.norm() as f32, 1.));
                fan.set_rotation(offset[1].atan2(offset[0]));
            }
        } else {
            log::warn!("Fan {} is not a Control node.", unsafe { fan.get_name() }.to_string());
        }
    }

    pub fn align_fans(&self, owner: Node2D) {
        let (cfg, data) = (&self.cfg, self.data.as_ref());
        let from = conv::g_to_na64(unsafe { owner.get_global_position() });
        let (upper, lower) = unsafe {
            (owner.get_node(cfg.upper_fan.new_ref()), owner.get_node(cfg.lower_fan.new_ref()))
        };
        if let Some(data) = data {
            let bounds = data.possible_angle_offsets(cfg);
            let (start, end) = bounds.into_inner();
            let to_aim = data.pos - from;
            upper.map(|upper| self.align_single_fan(upper, from, to_aim, start));
            lower.map(|lower| self.align_single_fan(lower, from, to_aim, end));
        }
    }

    fn set_fan_visibility(&self, owner: Node, should_be_visible: bool) {
        if should_be_visible {
            log::info!("Aim fan should be visible.");
        } else {
            log::info!("Aim fan should be invisible.");
        }
        let (upper, lower) = unsafe {
            (owner.get_node(self.cfg.upper_fan.new_ref()), owner.get_node(self.cfg.lower_fan.new_ref()))
        };
        unsafe {
            upper.and_then(|upper| upper.cast::<Control>())
                .tap_none(|| log::warn!("No lower fan found for object {}!", owner.get_name().to_string()))
                .map(|mut upper| upper.set_visible(should_be_visible));
            lower.and_then(|lower| lower.cast::<Control>())
                .tap_none(|| log::warn!("No lower fan found for object {}!", owner.get_name().to_string()))
                .map(|mut lower| lower.set_visible(should_be_visible));
        }
    }

    pub fn aim_at(&mut self, owner: Node, pos: na::Vector2<f64>) {
        let (cfg, data) = (&self.cfg, self.data.as_mut());
        if let Some(data) = data {
            data.aim_at(cfg, pos);
        } else {
            self.data = Some(Data::with_aim(&self.cfg, pos));
        }
        self.set_fan_visibility(owner, true);
        if let Some(owner) = unsafe { owner.cast() } {
            self.align_fans(owner);
        }
    }

    pub fn narrow_aim(&mut self, owner: Node, delta: Duration) {
        let (cfg, data) = (&self.cfg, self.data.as_mut());
        let data = if let Some(data) = data { data } else { return; };
        if data.step_time(cfg, delta) {
            self.set_fan_visibility(owner, false);
            self.data = None;
        } else {
            unsafe { owner.cast() }.map(|owner| self.align_fans(owner));
        }
    }

    pub fn reset(&mut self, owner: Node) {
        // TODO disable any nodes that need to be disabled here.
        self.set_fan_visibility(owner, false);
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

    fn calc_dir(&self, from: na::Vector2<f64>) -> Option<na::Vector2<f64>> {
        let (data, cfg) = (self.data.as_ref(), &self.cfg);
        let data = data.tap_none(|| log::warn!("Attempted to attack when not aiming... how?"))?;
        Some(data.true_dir(cfg, from))
    }

    pub fn attack(&mut self, from: na::Vector2<f64>, owner: Node, dmg: f64) {
        (|| {
            // Unpack data & cache.
            {
                let data = self.data.as_mut()
                    .tap_none(|| log::warn!("Attempted to attack when not aiming... how?"))?;
                // Modify stage
                data.stage = Stage::Cooldown;
                self.set_fan_visibility(owner, false);
            }
            let cache = self.cache.as_ref()
                .tap_none(|| log::warn!("Failed to load cache earlier."))?;
            // Init projectile
            let projectile = cache.projectile_scene
                .lock().ok().tap_none(|| log::warn!("Failed to lock loaded projectile scene."))?
                .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                .tap_none(|| log::warn!("Could not instance projectile scene."))?;
            let direction = self.calc_dir(from)
                .tap_none(|| log::warn!("Could not determine correct direction."))?;
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
