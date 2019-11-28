use nalgebra as na;
use gdnative::{
    self as godot,
    GodotString,
    init::{Property, PropertyHint, ClassBuilder,},
    KinematicBody2D,
    NativeClass,
    Node,
    Object,
    user_data::MutexData,
};
use crate::{
    util::{conv, Group},
    systems::{
        self,
        EditorCfg,
        health::System as HealthSys,
    },
};

pub struct Cfg {
    velocity: f64,
    max_bounces: u64,
}

impl Cfg {
    const VELOCITY: f64 = 10.;
    const MAX_BOUNCES: u64 = 0;
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            velocity: Self::VELOCITY,
            max_bounces: Self::MAX_BOUNCES,
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
            name: "velocity",
            default: Self::VELOCITY,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).velocity,
            setter: move |this: &mut T, vel| get_mut(this).velocity = vel,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "max_bounces",
            default: Self::MAX_BOUNCES,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).max_bounces,
            setter: move |this: &mut T, max| get_mut(this).max_bounces = max,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Default)]
pub struct Normal {
    cfg: Cfg,
    dir: godot::Vector2,
    dmg: f64,
}

impl godot::NativeClass for Normal {
    type Base = KinematicBody2D;
    type UserData = MutexData<Normal>;

    fn class_name() -> &'static str {
        "NormalProjectile"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        Cfg::register_properties(builder, |this| &this.cfg, |this| &mut this.cfg);
    }
}

#[methods]
impl Normal {
    fn _init(_owner: KinematicBody2D) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&self, mut owner: KinematicBody2D) {
        unsafe { owner.add_to_group("projectile".into(), false) };
        log::info!("Hello from projectile.")
    }

    fn inflict(&self, _owner: KinematicBody2D, target: Object) {
        if let Some(target) = unsafe { target.cast::<Node>() } {
            log::info!("Projectile collided with {:?}.", unsafe { target.get_name() });
            let groups = unsafe { target.get_groups() };
            if groups.contains(&(&GodotString::from("enemy")).into()) || groups.contains(&(&GodotString::from("switch")).into()) {
                HealthSys::call_damage(unsafe { target.to_object() }, self.dmg);
            }
        }
    }

    #[export]
    fn _physics_process(&mut self, mut owner: KinematicBody2D, _delta: f64) {
        // TODO make velocity a param.
        if let Some(collision) = unsafe { owner.move_and_collide(self.dir * (self.cfg.velocity as f32), true, true, false) } {
            if let Some(collider) = collision.get_collider() {
                log::info!("Inflicting damage!");
                self.inflict(owner, collider);
            }
            unsafe { owner.queue_free(); }
        }
    }
}

impl Normal {
    pub fn init_instance(
        &mut self,
        mut owner: KinematicBody2D,
        pos: na::Vector2<f64>,
        dir: na::Vector2<f64>,
        dmg: f64,
    ) {
        let pos = conv::na64_to_g(pos);
        let dir = conv::na64_to_g(dir);
        unsafe { owner.set_position(pos); }
        self.dir = dir;
        unsafe { owner.set_global_position(dir) };
        self.dmg = dmg;
    }
}

#[derive(Default)]
pub struct Charged {
    cfg: Cfg,
    dir: godot::Vector2,
    dmg: f64,
    remaining_bounces: u64,
}

impl NativeClass for Charged {
    type Base = KinematicBody2D;
    type UserData = MutexData<Charged>;

    fn class_name() -> &'static str {
        "ChargedProjectile"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        Cfg::register_properties(builder, |this| &this.cfg, |this| &mut this.cfg);
    }
}

#[methods]
impl Charged {
    fn _init(_owner: KinematicBody2D) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&self, owner: KinematicBody2D) {
        Group::Projectile.add_node(unsafe { owner.to_node() });
        log::info!("Hello from projectile.")
    }

    fn inflict(&self, _owner: KinematicBody2D, target: Object) {
        if let Some(target) = unsafe { target.cast::<Node>() } {
            log::info!("Projectile collided with {:?}.", unsafe { target.get_name() });
            if Group::Enemy.has_node(target) || Group::Switch.has_node(target) {
                HealthSys::call_damage(unsafe { target.to_object() }, self.dmg);
            }
        }
    }

    fn bounce(&mut self, mut owner: KinematicBody2D, target: Object, normal: na::Vector2<f64>) {
        if let Some(target) = unsafe { target.cast::<Node>() } {
            log::info!("Projectile collided with {:?}.", unsafe { target.get_name() });
            if !Group::Enemy.has_node(target) && Group::Switch.has_node(target) {
                let cos = conv::g_to_na64(self.dir).dot(&normal);
                let reflect_vec = 2. * (cos * normal);
                self.dir -= conv::na64_to_g(reflect_vec);
            } else {
                unsafe { owner.queue_free(); }
            }
        }
    }

    #[export]
    fn _physics_process(&mut self, mut owner: KinematicBody2D, _delta: f64) {
        // TODO make velocity a param.
        if let Some(collision) = unsafe {
            owner.move_and_collide(
                self.dir * (self.cfg.velocity as f32),
                true, true, false
            )
        } {
            if let Some(collider) = collision.get_collider() {
                log::info!("Inflicting damage!");
                self.inflict(owner, collider);
                self.bounce(owner, collider, conv::g_to_na64(collision.get_normal()));
            } else {
                unsafe { owner.queue_free(); }
            }
        }
    }
}

impl Charged {
    pub fn init_instance(
        &mut self,
        mut owner: KinematicBody2D,
        pos: na::Vector2<f64>,
        dir: na::Vector2<f64>,
        dmg: f64,
    ) {
        let pos = conv::na64_to_g(pos);
        let dir = conv::na64_to_g(dir);
        unsafe { owner.set_position(pos); }
        self.dir = dir;
        unsafe { owner.set_global_position(dir) };
        self.dmg = dmg;
        self.remaining_bounces = self.cfg.max_bounces;
    }
}
