use gdnative::{
    Area2D,
    GodotString,
    NativeClass,
    Node,
    init::{ClassBuilder, Property, PropertyHint},
    user_data::MutexData,
    StringArray,
    Variant,
};
use std::time::Duration;

use crate::{
    util::{
        Direction,
        Group,
    },
    systems::{
        self,
        EditorCfg,
        health::System as HealthSys,
    },
};

#[derive(Debug)]
struct Cfg {
    next_attack: Option<u64>,
    dmg: f64,
    animation_duration: Duration,
    hit_duration: Duration,
    cooldown_duration: Duration,
    target: Vec<GodotString>,
}

impl Cfg {
    const NEXT_ATTACK: Option<u64> = None;
    const HIT_DURATION: Duration = Duration::from_millis(100);
    const COOLDOWN_DURATION: Duration = Duration::from_millis(100);
    const ANIMATION_DURATION: Duration = Duration::from_millis(500);
    const DMG: f64 = 10.;
    // TODO switch default to player later.
    const TARGET: &'static [&'static str] = &["enemy"];
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            next_attack: Self::NEXT_ATTACK,
            cooldown_duration: Self::COOLDOWN_DURATION,
            hit_duration: Self::HIT_DURATION,
            animation_duration: Self::ANIMATION_DURATION,
            dmg: Self::DMG,
            target: Self::TARGET.iter().map(|s| s.into()).collect(),
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
            name: "next_attack",
            default: -1,
            hint: PropertyHint::Range {
                range: (-1.)..(std::i32::MAX as f64),
                step: 1.,
                slider: false,
            },
            getter: move |this: &T| get(this).next_attack.map(|i| i as i64).unwrap_or(-1),
            setter: move |this: &mut T, next_attack| get_mut(this).next_attack = if next_attack < 0 {
                None
            } else {
                Some(next_attack as u64)
            },
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "damage",
            default: Self::DMG,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).dmg,
            setter: move |this: &mut T, dmg| get_mut(this).dmg = dmg,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "animation_duration",
            default: Self::ANIMATION_DURATION.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).animation_duration.as_millis() as u64,
            setter: move |this: &mut T, d| get_mut(this).animation_duration = Duration::from_millis(d),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "hit_duration",
            default: Self::HIT_DURATION.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).hit_duration.as_millis() as u64,
            setter: move |this: &mut T, d| get_mut(this).hit_duration = Duration::from_millis(d),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "cooldown",
            default: Self::COOLDOWN_DURATION.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).cooldown_duration.as_millis() as u64,
            setter: move |this: &mut T, d| get_mut(this).cooldown_duration = Duration::from_millis(d),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "targets",
            default: (|| {
                let mut buf = StringArray::new();
                for &target in Self::TARGET {
                    buf.push(&target.into());
                }
                buf
            })(),
            hint: Group::full_property_hint(),
            getter: move |this: &T| {
                let mut buf = StringArray::new();
                for target in get(this).target.iter() {
                    buf.push(target);
                }
                buf
            },
            setter: move |this: &mut T, targets: StringArray| {
                let mut buf = Vec::with_capacity(targets.len() as usize);
                for i in 0..targets.len() {
                    buf.push(targets.get(i));
                }
                get_mut(this).target = buf;
            },
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Debug)]
struct Data {
    remaining_cooldown_duration: Duration,
    remaining_animating_duration: Duration,
    remaining_hit_duration: Duration,
}

impl Data {
    const ZERO: Duration = Duration::from_millis(0);

    fn is_finished(&self) -> bool {
        self.remaining_cooldown_duration == Self::ZERO
        && self.remaining_animating_duration == Self::ZERO
        && self.remaining_hit_duration == Self::ZERO
    }

    fn step_time(&mut self, delta: Duration) {
        if self.remaining_hit_duration > delta {
            self.remaining_hit_duration -= delta;
        } else {
            self.remaining_hit_duration = Self::ZERO;
        }
        if self.remaining_cooldown_duration > delta {
            self.remaining_cooldown_duration -= delta;
        } else {
            self.remaining_cooldown_duration = Self::ZERO;
        }
        if self.remaining_animating_duration > delta {
            self.remaining_animating_duration -= delta;
        } else {
            self.remaining_animating_duration = Self::ZERO;
        }
    }
}

#[derive(Default, Debug)]
pub struct Attack {
    cfg: Cfg,
    data: Option<Data>,
}

impl NativeClass for Attack {
    type Base = Area2D;
    type UserData = MutexData<Attack>;

    fn class_name() -> &'static str {
        "Attack"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        Cfg::register_properties(builder, |this| &this.cfg, |this| &mut this.cfg);
    }
}

impl Attack {
    fn get_hit_objects(&self, owner: Area2D) -> Vec<Node> {
        let bodies = unsafe { owner.get_overlapping_bodies() };
        bodies
            .iter()
            .flat_map(|s| s.try_to_object())
            .collect()
    }
    fn can_hit(&self, target: &Node) -> bool {
        let data = if let Some(data) = &self.data {
            data
        } else {
            return false;
        };

        if data.remaining_hit_duration == Duration::from_millis(0) {
            return false;
        }

        let target_groups = unsafe { target.get_groups() };
        'match_loop: for body_group in target_groups.iter().flat_map(|s| s.try_to_godot_string()) {
            for target in &self.cfg.target {
                if target == &body_group {
                    return true;
                }
            }
        }
        false
    }
}

#[methods]
impl Attack {
    fn _init(mut owner: Area2D) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&self, _owner: Area2D) {
        log::info!("Hello from attacks!")
    }

    #[export]
    fn _physics_process(&mut self, owner: Area2D, delta: f64) {
        let delta = Duration::from_secs_f64(delta);

        if self.data.is_none() {
            return;
        };

        // Hit
        let hit: Vec<_> = self.get_hit_objects(owner)
            .into_iter()
            .filter(|obj| self.can_hit(obj))
            .collect();
        for obj in hit.clone() {
            HealthSys::call_damage(unsafe { obj.to_object() }, self.cfg.dmg);
        }

        // Check if attack is completed.
        if self.data.as_mut().map_or(false, |data| {
            data.step_time(delta);
            data.is_finished()
        }) {
            self.data = None;
            let mut owner = unsafe { owner.to_canvas_item() };
            unsafe { owner.set_visible(false) };
        }
    }

    #[export]
    fn execute(&mut self, mut owner: Area2D, dir: Option<f64>) {
        use std::f64::consts::PI;
        if let Some(dir) = dir {
            self.data = Some(Data {
                remaining_cooldown_duration: self.cfg.cooldown_duration,
                remaining_animating_duration: self.cfg.animation_duration,
                remaining_hit_duration: self.cfg.hit_duration,
            });
            // Orientation is upside down in screen space.
            // [0, pi] -> [2pi, pi] | [pi, 2pi] -> [pi, 0]
            let dir = 2. * PI - dir;
            unsafe {
                owner.set_global_rotation(dir);
                let mut canvas_item_owner = owner.to_canvas_item();
                canvas_item_owner.set_visible(true);
            }
        } else {
            self.data = Some(Data {
                remaining_cooldown_duration: self.cfg.cooldown_duration,
                remaining_animating_duration: Duration::from_millis(0),
                remaining_hit_duration: Duration::from_millis(0),
            });
        }
    }

    #[export]
    fn next_attack(&self, _owner: Area2D) -> Option<u64> {
        self.cfg.next_attack
    }

    #[export]
    fn cooldown(&self, _owner: Area2D) -> u64 {
        self.cfg.cooldown_duration.as_millis() as u64
    }
}

impl Attack {
    pub fn call_next_attack(owner: &mut Area2D) -> Option<u64> {
        // Next attack
        let next_atk_fn = "next_attack".into();
        unsafe { owner.call(next_atk_fn, &[]).try_to_u64() }
    }
    pub fn call_cooldown(owner: &mut Area2D) -> Option<Duration> {
        // Cooldown until can use another attack.
        let cooldown_fn = "cooldown".into();
        let cooldown = unsafe {
            owner.call(cooldown_fn, &[])
        }.try_to_u64()?;
        Some(Duration::from_millis(cooldown))
    }
    pub fn call_execute(mut owner: Area2D, dir: Direction) {
        log::info!("Attempting to attack in the {:?} direction.", dir);
        unsafe {
            owner.call("execute".into(), &[
                dir.to_radians().map_or_else(|| Variant::new(), |r| Variant::from_f64(r)),
            ]);
        }
    }
}
