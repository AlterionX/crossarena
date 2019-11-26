use nalgebra as na;
use gdnative::{
    NativeClass,
    init::{ClassBuilder, Property, PropertyHint,},
    user_data::MutexData,
};
use std::time::Duration;
use crate::{
    systems::{self, EditorCfg},
    util::Direction,
};

#[derive(Debug)]
pub struct Cfg {
    pub invincibility: Duration,
    pub duration: Duration,
    pub cooldown: Duration,
    pub slowdown: f64,
    pub speed: f64,
    pub chain: u8,
}

impl Cfg {
    const SPEED: f64 = 500.;
    const CHAIN: u8 = 3;
    const DURATION: Duration = Duration::from_millis(200);
    const COOLDOWN: Duration = Duration::from_millis(50);
    const SLOWDOWN: f64 = 15.;
    const INVINCIBILITY: Duration = Duration::from_millis(100);
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            invincibility: Self::INVINCIBILITY,
            duration: Self::DURATION,
            cooldown: Self::COOLDOWN,
            slowdown: Self::SLOWDOWN,
            speed: Self::SPEED,
            chain: Self::CHAIN,
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
            name: "dash/invinciblity",
            default: Self::INVINCIBILITY.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).invincibility.as_millis() as u64,
            setter: move |this: &mut T, inv| get_mut(this).invincibility = Duration::from_millis(inv),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "dash/slow_speed",
            default: Self::SLOWDOWN,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).slowdown,
            setter: move |this: &mut T, spd| get_mut(this).slowdown = spd,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "dash/dash_speed",
            default: Self::SPEED,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).speed,
            setter: move |this: &mut T, spd| get_mut(this).speed = spd,
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "dash/duration",
            default: Self::DURATION.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).duration.as_millis() as u64,
            setter: move |this: &mut T, duration| get_mut(this).duration = Duration::from_millis(duration),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "dash/cooldown",
            default: Self::COOLDOWN.as_millis() as u64,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).cooldown.as_millis() as u64,
            setter: move |this: &mut T, cooldown| get_mut(this).cooldown = Duration::from_millis(cooldown as u64),
            usage: *systems::DEFAULT_USAGE,
        });
        let get = get_proto.clone();
        let get_mut = get_mut_proto.clone();
        builder.add_property(Property {
            name: "dash/consecutive_dashes",
            default: Self::CHAIN,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).chain,
            setter: move |this: &mut T, chain| get_mut(this).chain = chain,
            usage: *systems::DEFAULT_USAGE,
        });
    }
}

pub struct Data {
    pub count: u8,
    pub since_start: Duration,
    pub dir: Direction,
}

impl Data {
    pub fn first_dash(d: Direction) -> Self {
        Self {
            count: 0,
            since_start: Duration::from_secs(0),
            dir: d,
        }
    }
    pub fn chained_dash(&self, d: Direction) -> Self {
        Self {
            count: self.count + 1,
            since_start: Duration::from_secs(0),
            dir: d,
        }
    }
}

#[derive(Default)]
pub struct System {
    pub cfg: Cfg,
    pub data: Option<Data>,
}

impl System {
    pub fn dash(&mut self, facing_dir: Direction) {
        if let Some(dashing) = &mut self.data {
            // Already dashing, so change direction and advance dash count.
            if dashing.count < self.cfg.chain {
                let chained_dash = dashing.chained_dash(facing_dir);
                log::info!("Begin chained dash number {}.", chained_dash.count);
                self.data = Some(chained_dash);
            } else {
                log::info!("Overdrafted dash. Forced slowdown.");
                dashing.since_start = Duration::from_secs(0);
            }
        } else {
            log::info!("Begin dash.");
            // Not yet dashing. Begin to dash.
            self.data = Some(Data::first_dash(facing_dir));
        }
    }
    pub fn calc_vel(&mut self, delta: Duration) -> Option<na::Vector2<f64>> {
        // TODO Change `to_na_vec` to `into` when able to.
        let data = self.data.as_mut()?;
        data.since_start += delta;
        Some(data.dir.to_na_vec() * if self.cfg.chain <= data.count || data.since_start > self.cfg.duration {
            if data.since_start > self.cfg.duration + self.cfg.cooldown {
                self.data = None;
            }
            self.cfg.slowdown
        } else {
            self.cfg.speed
        })
    }
    pub fn is_dashing(&self) -> bool {
        self.data.is_some()
    }
    pub fn invincibility(&self) -> Duration {
        self.cfg.invincibility
    }
}
