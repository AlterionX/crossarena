use nalgebra as na;
use gdnative::{
    Control,
    GodotString,
    init::{ClassBuilder, Property, PropertyHint,},
    Instance,
    NativeClass,
    Node,
    Node2D,
    NodePath,
    PackedScene,
    ResourceLoader,
    Texture,
    TextureRect,
    user_data::MutexData,
};
use tap::TapOptionOps;
use boolinator::Boolinator;
use rand::distributions::{Distribution, Uniform};
use std::{ops::RangeInclusive, sync::{Arc, Mutex}, time::Duration};
use crate::{
    util::{conv, Direction, common_matrices as common_mats},
    systems::{self, System as SysTrait, EditorCfg},
    entity::{NormalProjectile, ChargedProjectile},
};

#[derive(Debug, PartialEq)]
pub struct Cfg {
    max_aim_time: Duration,
    charge_time: Duration,
    min_aim_time: Duration,
    cooldown_time: Duration,

    normal_projectile: GodotString,
    charged_projectile: GodotString,

    dmg: f64,
    aim_range_off_rot: f64,
    world: NodePath,

    upper_fan: NodePath,
    lower_fan: NodePath,
    ufan_normal_texture: GodotString,
    lfan_normal_texture: GodotString,
    ufan_charged_texture: GodotString,
    lfan_charged_texture: GodotString,
    fan_offset: f64,

    walk_speed: f64,
}

impl Cfg {
    const MAX_AIM: Duration = Duration::from_millis(3000);
    const CHARGE: Duration = Duration::from_millis(100);
    const MIN_AIM: Duration = Duration::from_millis(100);
    const COOLDOWN: Duration = Duration::from_millis(100);

    const DMG: f64 = 10.;
    const AIM_OFF: f64 = std::f64::consts::FRAC_PI_2;
    const WORLD: &'static str = "..";

    const NORMAL_PROJECTILE_PATH: &'static str = "res://projectile.tscn";
    const CHARGED_PROJECTILE_PATH: &'static str = "res://charged_projectile.tscn";

    const UPPER_FAN: &'static str = "LowerFan";
    const LOWER_FAN: &'static str = "UpperFan";
    const UPPER_FAN_NORMAL_TEX_PATH: &'static str = "res://ufan_normal.tres";
    const LOWER_FAN_NORMAL_TEX_PATH: &'static str = "res://lfan_normal.tres";
    const UPPER_FAN_CHARGED_TEX_PATH: &'static str = "res://ufan_charged.tres";
    const LOWER_FAN_CHARGED_TEX_PATH: &'static str = "res://lfan_charged.tres";
    const FAN_OFFSET: f64 = 50.;

    const WALK_SPEED: f64 = 20.;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            max_aim_time: Self::MAX_AIM,
            charge_time: Self::CHARGE,
            min_aim_time: Self::MIN_AIM,
            cooldown_time: Self::COOLDOWN,

            dmg: Self::DMG,
            aim_range_off_rot: Self::AIM_OFF,
            world: NodePath::from_str(Self::WORLD),

            normal_projectile: Self::NORMAL_PROJECTILE_PATH.into(),
            charged_projectile: Self::CHARGED_PROJECTILE_PATH.into(),

            lower_fan: NodePath::from_str(Self::UPPER_FAN),
            upper_fan: NodePath::from_str(Self::LOWER_FAN),
            ufan_normal_texture: Self::UPPER_FAN_NORMAL_TEX_PATH.into(),
            lfan_normal_texture: Self::LOWER_FAN_NORMAL_TEX_PATH.into(),
            ufan_charged_texture: Self::UPPER_FAN_CHARGED_TEX_PATH.into(),
            lfan_charged_texture: Self::LOWER_FAN_CHARGED_TEX_PATH.into(),
            fan_offset: Self::FAN_OFFSET,

            walk_speed: Self::WALK_SPEED,
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
            name: "aim/charge_time",
            default: Self::CHARGE.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).charge_time.as_millis() as u64,
            setter: move |this: &mut T, charge| get_mut(this).charge_time = Duration::from_millis(charge as u64),
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
            name: "aim/charged_projectile_scene",
            default: Self::CHARGED_PROJECTILE_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).charged_projectile.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).charged_projectile = path,
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
            name: "aim/ufan_normal",
            default: Self::UPPER_FAN_NORMAL_TEX_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).ufan_normal_texture.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).ufan_normal_texture = path,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/lfan_normal",
            default: Self::LOWER_FAN_NORMAL_TEX_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).lfan_normal_texture.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).lfan_normal_texture = path,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/ufan_charged",
            default: Self::UPPER_FAN_CHARGED_TEX_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).ufan_charged_texture.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).ufan_charged_texture = path,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "aim/lfan_Charged",
            default: Self::LOWER_FAN_CHARGED_TEX_PATH.into(),
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).lfan_charged_texture.new_ref(),
            setter: move |this: &mut T, path: GodotString| get_mut(this).lfan_charged_texture = path,
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
pub struct Cache {
    projectile_scene: Arc<Mutex<PackedScene>>,
    charged_projectile_scene: Arc<Mutex<PackedScene>>,
    upper_fan_material_normal: Arc<Mutex<Texture>>,
    lower_fan_material_normal: Arc<Mutex<Texture>>,
    upper_fan_material_charged: Arc<Mutex<Texture>>,
    lower_fan_material_charged: Arc<Mutex<Texture>>,
}

impl Cache {
    fn load_with(cfg: &Cfg) -> Option<Self> {
        let mut loader = ResourceLoader::godot_singleton();
        let normal_scene = loader
            .load(cfg.normal_projectile.new_ref(), "PackedScene".into(), false)
            .and_then(|loaded| loaded.cast::<PackedScene>())?;
        let charged_scene = loader
            .load(cfg.charged_projectile.new_ref(), "PackedScene".into(), false)
            .and_then(|loaded| loaded.cast::<PackedScene>())?;
        let ufan_normal = loader
            .load(cfg.ufan_normal_texture.new_ref(), "Texture".into(), false)
            .and_then(|loaded| loaded.cast::<Texture>())?;
        let lfan_normal = loader
            .load(cfg.lfan_normal_texture.new_ref(), "Texture".into(), false)
            .and_then(|loaded| loaded.cast::<Texture>())?;
        let ufan_charged = loader
            .load(cfg.ufan_charged_texture.new_ref(), "Textuee".into(), false)
            .and_then(|loaded| loaded.cast::<Texture>())?;
        let lfan_charged = loader
            .load(cfg.lfan_charged_texture.new_ref(), "Texture".into(), false)
            .and_then(|loaded| loaded.cast::<Texture>())?;
        Some(Self {
            projectile_scene: Arc::new(Mutex::new(normal_scene)),
            charged_projectile_scene: Arc::new(Mutex::new(charged_scene)),
            upper_fan_material_normal: Arc::new(Mutex::new(ufan_normal)),
            lower_fan_material_normal: Arc::new(Mutex::new(lfan_normal)),
            upper_fan_material_charged: Arc::new(Mutex::new(ufan_charged)),
            lower_fan_material_charged: Arc::new(Mutex::new(lfan_charged)),
        })
    }

    fn fan_tex(&self, is_upper: bool, is_charged: bool) -> Option<Texture> {
        let tex = match (is_upper, is_charged) {
            (true, false) => &self.upper_fan_material_normal,
            (false, false) => &self.lower_fan_material_normal,
            (true, true) => &self.upper_fan_material_charged,
            (false, true) => &self.lower_fan_material_charged,
        };
        tex.lock().ok().map(|tex| tex.new_ref())
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

enum TimeEvent {
    ChargedUp,
    FinishedCooldown,
}

#[derive(Debug, PartialEq)]
pub struct Data {
    stage: Stage,
    pos: na::Vector2<f64>,
    time_to_aim: Duration,
    time_to_charge: Duration,
    cooling_down: Duration,
}

impl Data {
    fn with_aim(cfg: &Cfg, pos: na::Vector2<f64>) -> Self {
        Self {
            pos,
            stage: Stage::WarmUp,
            time_to_aim: cfg.max_aim_time,
            time_to_charge: Duration::from_millis(0),
            cooling_down: Duration::from_millis(0),
        }
    }

    fn adv_time(&mut self, mut time_to_deduct: Duration) {
        const ZERO: Duration = Duration::from_millis(0);
        if time_to_deduct != ZERO && self.time_to_aim != ZERO {
            if self.time_to_aim < time_to_deduct {
                time_to_deduct -= self.time_to_aim;
                self.time_to_aim = ZERO;
            } else {
                self.time_to_aim -= time_to_deduct;
                time_to_deduct = ZERO;
            }
        }
        if time_to_deduct != ZERO && self.time_to_charge != ZERO {
            if self.time_to_charge < time_to_deduct {
                time_to_deduct -= self.time_to_charge;
                self.time_to_charge = ZERO;
            } else {
                self.time_to_charge -= time_to_deduct;
                // time_to_deduct = ZERO;
            }
        }
    }

    fn rewind_time(&mut self, cfg: &Cfg, mut time_to_deduct: Duration) {
        const ZERO: Duration = Duration::from_millis(0);
        if time_to_deduct != ZERO && self.time_to_charge != cfg.charge_time {
            if (self.time_to_charge + time_to_deduct) < cfg.charge_time {
                self.time_to_charge += time_to_deduct;
                time_to_deduct = ZERO;
            } else {
                time_to_deduct -= self.time_to_charge;
                self.time_to_charge = cfg.charge_time;
            }
        }
        if time_to_deduct != ZERO && self.time_to_aim != cfg.max_aim_time {
            if (self.time_to_aim + time_to_deduct) < cfg.max_aim_time {
                self.time_to_aim += time_to_deduct;
                // time_to_deduct = ZERO;
            } else {
                time_to_deduct -= self.time_to_aim;
                self.time_to_aim = cfg.max_aim_time;
            }
        }
    }

    fn aim_at(&mut self, cfg: &Cfg, pos: na::Vector2<f64>) {
        self.pos = pos;
        if self.stage.is_cooldown() {
            // Hot start
            self.rewind_time(cfg, cfg.min_aim_time);
            self.stage = Stage::WarmUp;
            self.cooling_down = Duration::from_millis(0);
        }
    }

    fn step_time(&mut self, cfg: &Cfg, delta: Duration) -> Option<TimeEvent> {
        match self.stage {
            Stage::WarmUp => {
                self.adv_time(delta);
                (self.time_to_charge == Duration::from_millis(0)).as_some(TimeEvent::ChargedUp)
            },
            Stage::Cooldown => {
                self.cooling_down += delta;
                (self.cooling_down >= cfg.cooldown_time).as_some(TimeEvent::FinishedCooldown)
            },
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

    fn is_charged(&self) -> bool {
        self.time_to_charge == Duration::from_millis(0)
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

    fn set_single_fan_charged(&self, mut fan: TextureRect, cache: &Cache, is_upper: bool, is_charged: bool) {
        let texture = cache.fan_tex(is_upper, is_charged);
        unsafe { fan.set_texture(texture) };
    }

    pub fn set_fan_charged(&self, owner: Node, is_charged: bool) {
        let (cfg, cache, _) = self.view();
        if is_charged {
            log::info!("Fan should be charged.");
        } else {
            log::info!("Fan should not be charged.");
        }
        let (upper, lower) = unsafe { (
            owner.get_node(cfg.upper_fan.new_ref()),
            owner.get_node(cfg.lower_fan.new_ref()),
        ) };
        cache.map(move |cache| unsafe {
            upper.and_then(|upper| upper.cast::<TextureRect>())
                .tap_none(|| log::warn!("No lower fan found for object {}!", owner.get_name().to_string()))
                .map(|upper| self.set_single_fan_charged(upper, cache, true, is_charged));
            lower.and_then(|lower| lower.cast::<TextureRect>())
                .tap_none(|| log::warn!("No lower fan found for object {}!", owner.get_name().to_string()))
                .map(|lower| self.set_single_fan_charged(lower, cache, false, is_charged));
        });
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
        let (cfg, _, data) = self.view();
        let data = if let Some(data) = data { data } else { return; };
        let from = conv::g_to_na64(unsafe { owner.get_global_position() });
        let (upper, lower) = unsafe { (
            owner.get_node(cfg.upper_fan.new_ref()),
            owner.get_node(cfg.lower_fan.new_ref()),
        ) };
        let (start, end) = data.possible_angle_offsets(cfg).into_inner();
        let to_aim = data.pos - from;
        upper.map(|upper| self.align_single_fan(upper, from, to_aim, start));
        lower.map(|lower| self.align_single_fan(lower, from, to_aim, end));
    }

    fn set_fan_visibility(&self, owner: Node, should_be_visible: bool) {
        if should_be_visible {
            log::info!("Aim fan should be visible.");
        } else {
            log::info!("Aim fan should be invisible.");
        }
        let (upper, lower) = unsafe { (
            owner.get_node(self.cfg.upper_fan.new_ref()),
            owner.get_node(self.cfg.lower_fan.new_ref()),
        ) };
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
        { // Data manipulation.
            let (cfg, _, data) = self.view_mut();
            if let Some(data) = data {
                data.aim_at(cfg, pos);
            } else {
                self.data = Some(Data::with_aim(cfg, pos));
            }
        }
        // Fan manipulation.
        let is_charged = self.data.as_ref().map(|data| data.is_charged()).unwrap_or(false);
        self.set_fan_charged(owner, is_charged);
        self.set_fan_visibility(owner, true);
        if let Some(owner) = unsafe { owner.cast() } {
            self.align_fans(owner);
        }
    }

    pub fn narrow_aim(&mut self, owner: Node, delta: Duration) {
        let cooldown_finished = {
            let (cfg, _, data) = self.view_mut();
            let data = if let Some(data) = data { data } else { return; };
            data.step_time(cfg, delta)
        };
        match cooldown_finished {
            Some(TimeEvent::FinishedCooldown) => {
                self.reset(owner)
            }
            Some(TimeEvent::ChargedUp) => {
                self.set_fan_charged(owner, true);
            }
            None => {
                unsafe { owner.cast() }.map(|owner| self.align_fans(owner));
            }
        }
    }

    pub fn reset(&mut self, owner: Node) {
        self.set_fan_charged(owner, false);
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

    pub fn shoot(&mut self, from: na::Vector2<f64>, owner: Node, dmg: f64) {
        self.set_fan_visibility(owner, false);
        let (cfg, cache, data) = match self.res_view_mut() {
            Ok(view) => view,
            Err(e) => {
                log::warn!(
                    "Error encountered when aim system of {} attempted to fire a shot:\n\t{}",
                    unsafe { owner.get_name() }.to_string(),
                    e,
                );
                return;
            }
        };
        data.stage = Stage::Cooldown;
        (|| {
            // Modify stage
            let direction = data.true_dir(cfg, from);
            // Init projectile
            let projectile_base = if data.is_charged() {
                &cache.charged_projectile_scene
            } else {
                &cache.projectile_scene
            };
            let projectile_base = projectile_base
                .lock().ok().tap_none(|| log::warn!("Failed to lock loaded projectile scene."))?
                .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                .tap_none(|| log::warn!("Could not instance projectile scene."))?;
            let projectile = unsafe { projectile_base.cast() }
                .tap_none(|| log::warn!(
                    "Provided normal projectile scene in node {} is not \
                    a KinematicBody2D, which is unexpected.",
                    unsafe { owner.get_name() }.to_string(),
                ))?;
            if data.is_charged() {
                let projectile: Instance<ChargedProjectile> = Instance::try_from_base(projectile)
                    .tap_none(|| log::warn!(
                        "Provided normal projectile scene does not have \
                        a `NormalProjectile` script attached  to the root node \
                        in node {}.",
                        unsafe { owner.get_name() }.to_string(),
                    ))?;
                projectile
                    .map_mut(|projectile, owner| projectile.init_instance(
                        owner,
                        from + 20. * direction,
                        direction,
                        dmg,
                    ))
                    .ok().tap_none(|| log::warn!(
                        "Failed to obtain lock on user data for projectile when {} attempted to fire!",
                        unsafe { owner.get_name() }.to_string(),
                    ))?;
            } else {
                let projectile: Instance<NormalProjectile> = Instance::try_from_base(projectile)
                    .tap_none(|| log::warn!(
                        "Provided normal projectile scene does not have \
                        a `NormalProjectile` script attached  to the root node \
                        in node {}.",
                        unsafe { owner.get_name() }.to_string(),
                    ))?;
                projectile
                    .map_mut(|projectile, owner| projectile.init_instance(
                        owner,
                        from + 20. * direction,
                        direction,
                        dmg,
                    ))
                    .ok().tap_none(|| log::warn!(
                        "Failed to obtain lock on user data for projectile when {} attempted to fire!",
                        unsafe { owner.get_name() }.to_string(),
                    ))?;
            }
            // Add to tree, and let it make its own way in the world.
            unsafe {
                owner
                    .get_node(cfg.world.new_ref())
                    .tap_none(|| log::warn!("Provided world to `Aim` system is incorrect."))?
                    .add_child(Some(projectile_base), false)
            }
            Some(())
        })();
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
