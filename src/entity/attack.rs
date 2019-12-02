use gdnative::{
    Area2D,
    GodotString,
    NativeClass,
    Node,
    NodePath,
    init::{ClassBuilder, Property, PropertyHint},
    user_data::MutexData,
    StringArray,
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
    max_hits: u64,
}

impl Cfg {
    const NEXT_ATTACK: Option<u64> = None;
    const HIT_DURATION: Duration = Duration::from_millis(300);
    const COOLDOWN_DURATION: Duration = Duration::from_millis(500);
    const ANIMATION_DURATION: Duration = Duration::from_millis(500);
    const DMG: f64 = 10.;
    // TODO switch default to player later.
    const TARGET: &'static [&'static str] = &["enemy"];
    const MAX_HITS: u64 = 1;
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
            max_hits: Self::MAX_HITS,
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
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "max_hits",
            default: Self::MAX_HITS,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).max_hits,
            setter: move |this: &mut T, max_hits| get_mut(this).max_hits = max_hits,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

#[derive(Debug)]
struct Data {
    remaining_cooldown_duration: Duration,
    remaining_animating_duration: Duration,
    remaining_hit_duration: Duration,
    // TODO make this a hash map
    hit_counts: Vec<(NodePath, u64)>,
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

    fn can_hit(&self) -> bool {
        self.remaining_hit_duration != Self::ZERO
    }

    fn add_hit_count(&mut self, obj: &Node) {
        let obj_path = unsafe { obj.get_path() };
        for (path, count) in self.hit_counts.iter_mut() {
            if *path == obj_path {
                *count += 1;
                return;
            }
        }
        self.hit_counts.push((obj_path, 1));
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

    fn is_in_target_groups(&self, target: &Node) -> bool {
        let target_groups = unsafe { target.get_groups() };
        for body_group in target_groups.iter().flat_map(|s| s.try_to_godot_string()) {
            for target in &self.cfg.target {
                if target == &body_group {
                    return true;
                }
            }
        }
        false
    }

    fn is_hit_less_than_max(&self, target: &Node) -> bool {
        (|| {
            let target_path = unsafe { target.get_path() };
            for (path, hits) in self.data.as_ref()?.hit_counts.iter() {
                if *path == target_path {
                    return Some(*hits < self.cfg.max_hits)
                }
            }
            None
        })().unwrap_or(true)
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

        self.is_in_target_groups(&target) && self.is_hit_less_than_max(&target)
    }
}

#[methods]
impl Attack {
    fn _init(_owner: Area2D) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&self, _owner: Area2D) {
        log::info!("Hello from attacks!")
    }

    #[export]
    fn _physics_process(&mut self, owner: Area2D, delta: f64) {
        let delta = Duration::from_secs_f64(delta);

        // Hit
        let hit = if let Some(data) = self.data.as_ref() {
            if data.can_hit() {
                self.get_hit_objects(owner)
                    .into_iter()
                    .filter(|obj| self.can_hit(obj))
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        if let Some(data) = self.data.as_mut() {
            for obj in hit {
                data.add_hit_count(&obj);
                HealthSys::call_damage(unsafe { obj.to_object() }, self.cfg.dmg);
            }

            data.step_time(delta);
            if data.is_finished() {
                self.reset(owner);
            }
        }
    }
}

impl Attack {
    pub fn execute(&mut self, mut owner: Area2D, dir: Direction) {
        use std::f64::consts::PI;
        let dir = dir.to_radians();
        if let Some(dir) = dir {
            self.data = Some(Data {
                remaining_cooldown_duration: self.cfg.cooldown_duration,
                remaining_animating_duration: self.cfg.animation_duration,
                remaining_hit_duration: self.cfg.hit_duration,
                hit_counts: vec![],
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
                hit_counts: vec![],
            });
        }
    }

    pub fn reset(&mut self, owner: Area2D) {
        self.data = None;
        let mut owner = unsafe { owner.to_canvas_item() };
        unsafe { owner.set_visible(false) };
    }

    pub fn next_attack(&self) -> Option<u64> {
        self.cfg.next_attack
    }

    pub fn cooldown(&self) -> Duration {
        self.cfg.cooldown_duration
    }
}
