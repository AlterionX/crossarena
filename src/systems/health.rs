use gdnative::{
    NativeClass,
    init::{ClassBuilder, Property, PropertyHint, PropertyUsage, Signal, SignalArgument,},
    user_data::MutexData,
    Variant,
    Object,
};
use std::time::{Duration};
use crate::systems::{self, System as SysTrait, EditorCfg};

#[derive(Debug)]
pub struct Cfg {
    max_hp: f64,
}

impl Cfg {
    const MAX_HP: f64 = 100.;

    const MAX_HP_SIGNAL: &'static str = "max_hp";
    const HP_SIGNAL: &'static str = "hp";
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg {
            max_hp: Self::MAX_HP,
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
            name: "health/max_hp",
            default: Self::MAX_HP,
            hint: PropertyHint::None,
            getter: move |this: &T| get(this).max_hp,
            setter: move |this: &mut T, path| get_mut(this).max_hp = path,
            usage: *systems::DEFAULT_USAGE,
        });
        builder.add_signal(Signal {
            name: Self::MAX_HP_SIGNAL,
            args: &[SignalArgument {
                name: "max_health",
                default: Variant::from_u64((100.0 as f64).to_bits()),
                hint: PropertyHint::None,
                usage: PropertyUsage::DEFAULT,
            }],
        });
        builder.add_signal(Signal {
            name: Self::HP_SIGNAL,
            args: &[SignalArgument {
                name: "health",
                default: Variant::from_u64((100.0 as f64).to_bits()),
                hint: PropertyHint::None,
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }
}

impl Cfg {
    pub fn broadcast_max_hp(&self, broadcaster: &mut Object) {
        unsafe {
            broadcaster.emit_signal("max_hp".into(), &[Variant::from_u64(self.max_hp.to_bits())]);
        }
    }
}

#[derive(Debug)]
pub struct Data {
    invincibility: Option<Duration>,
    hp: f64,
}

impl Data {
    fn process(&mut self, delta: Duration) {
        if let Some(inv) = self.invincibility.as_mut() {
            if *inv > delta {
                *inv -= delta;
            } else {
                self.invincibility = None;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct System {
    pub cfg: Cfg,
    cache: (),
    pub data: Option<Data>,
}

impl System {
    pub fn init(&mut self) {
        self.data = Some(Data {
            invincibility: None,
            hp: self.cfg.max_hp,
        });
    }
    pub fn broadcast_hp(&self, broadcaster: &mut Object) {
        let hp = self.data.as_ref().map(|d| d.hp).unwrap_or(0.).to_bits();
        let hp = Variant::from_u64(hp);
        unsafe {
            broadcaster.emit_signal(Cfg::HP_SIGNAL.into(), &[hp]);
        }
    }
    pub fn broadcast_max_hp(&self, broadcaster: &mut Object) {
        self.cfg.broadcast_max_hp(broadcaster);
    }

    pub fn process(&mut self, delta: Duration) {
        self.data.as_mut().map(|data| data.process(delta));
    }

    pub fn is_invincible(&self) -> bool {
        if let Some(data) = &self.data {
            data.invincibility.is_some()
        } else {
            false
        }
    }
    pub fn is_dead(&self) -> bool {
        self.data.is_none()
    }

    pub fn set_invincibility(&mut self, duration: Duration) {
        self.data.as_mut().map(|data| data.invincibility = Some(duration));
    }
    pub fn damage(&mut self, dmg: f64, mut to_notify: Option<Object>) -> f64 {
        if !self.is_invincible() {
            if let Some(data) = self.data.as_mut() {
                data.hp -= dmg;
                if data.hp <= 0. {
                    self.data = None;
                }
            }
        }
        if let Some(target) = to_notify.as_mut() {
            self.broadcast_hp(target);
        }
        self.data.as_ref().map_or(
            0.,
            |data| data.hp,
        )
    }

    pub fn get_max_hp(&self) -> u64 {
        self.cfg.max_hp as u64
    }
}

impl System {
    pub fn call_damage(mut target: Object, dmg: f64) {
        unsafe { target.call("damage".into(), &[Variant::from_f64(dmg)]) };
    }
}

impl SysTrait for System {
    type Cfg = Cfg;
    type Cache = ();
    type Data = Data;

    fn view(&self) -> (&Self::Cfg, Option<&Self::Cache>, Option<&Self::Data>) {
        (&self.cfg, Some(&()), self.data.as_ref())
    }
    fn view_mut(&mut self) -> (&mut Self::Cfg, Option<&mut Self::Cache>, Option<&mut Self::Data>) {
        (&mut self.cfg, Some(&mut self.cache), self.data.as_mut())
    }
}
