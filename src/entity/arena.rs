use nalgebra as na;
use gdnative::{
    self as godot,
    GodotString,
    Node,
    NodePath,
    Object,
    PackedScene,
    init::{Property, PropertyHint, PropertyUsage, Signal, SignalArgument},
    ResourceLoader,
    VariantArray,
    Variant,
};
use std::sync::{Arc, Mutex};
use tap::TapOptionOps;
use crate::entity::switch::Switch;

mod spawn;
use spawn::{Cfg as SpawnCfg, System as SpawnSystem};
mod wave;
use wave::*;

#[derive(Debug)]
struct Cfg {
    switch: GodotString,
    world: NodePath,
    player: NodePath,
    arena_dim: na::Vector2<f64>,
    arena_pos: na::Vector2<f64>,
}

impl Cfg {
    pub const WORLD: &'static str = "World";
    pub const SWITCH: &'static str = "res://switch/switch.tscn";
    pub const PLAYER: &'static str = "Player";
    pub const ARENA_DIM: [f64; 2] = [944., 520.];
    pub const ARENA_POS: [f64; 2] = [40., 40.];
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            world: NodePath::from_str(Self::WORLD),
            switch: Self::SWITCH.into(),
            player: Self::PLAYER.into(),
            arena_dim: na::Vector2::from_column_slice(&Self::ARENA_DIM),
            arena_pos: na::Vector2::from_column_slice(&Self::ARENA_POS),
        }
    }
}

#[derive(Debug)]
struct Cache {
    switch_scene: Arc<Mutex<PackedScene>>,
}

impl Cache {
    fn load_with(cfg: &Cfg) -> Option<Self> {
        log::info!("Loading Arena cache.");
        (|| {
            let enemy_scene_path: GodotString = cfg.switch.new_ref();
            let mut loader = ResourceLoader::godot_singleton();
            let loaded = loader.load(enemy_scene_path, "PackedScene".into(), true)
                .tap_none(|| log::info!("Failed to load scene from {:?}.", cfg.switch.to_string()))?;
            let loaded = loaded.cast()
                .tap_none(|| log::info!("Failed to cast instanced scene {:?}.", cfg.switch.to_string()))?;
            Some(Self {
                switch_scene: Arc::new(Mutex::new(loaded)),
            })
        })()
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

#[derive(Default)]
pub struct Arena {
    cfg: Cfg,
    cache: Option<Cache>,
    spawned_switch_path: Option<NodePath>,
    spawn_sys: SpawnSystem,
    spawn_count: u64,
    wave: Option<Wave>,
}

impl Arena {
    const ARENA_READY: &'static str = "arena_ready";
    const WAVE_NUMBER_CHANGED: &'static str = "wave_num_changed";
}

impl godot::NativeClass for Arena {
    type Base = godot::Node;
    type UserData = godot::user_data::MutexData<Arena>;

    fn class_name() -> &'static str {
        "Arena"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        let default_usage = PropertyUsage::SCRIPT_VARIABLE | PropertyUsage::STORAGE | PropertyUsage::EDITOR;
        builder.add_property(Property {
            name: "spawn/enemy_dir",
            default: SpawnCfg::DIR.to_string(),
            hint: PropertyHint::None,
            getter: |this: &Arena| (&this.spawn_sys.cfg.dir).into(),
            setter: |this: &mut Arena, dir| this.spawn_sys.cfg.dir = dir,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "spawn/listing",
            default: GodotString::from_str(SpawnCfg::CFG_FILE),
            hint: PropertyHint::None,
            getter: |this: &Arena| (&this.spawn_sys.cfg.cfg_file).into(),
            setter: |this: &mut Arena, dir: GodotString| this.spawn_sys.cfg.cfg_file = dir.to_string(),
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "world_root",
            default: NodePath::from_str(Cfg::WORLD),
            hint: PropertyHint::None,
            getter: |this: &Arena| this.cfg.world.new_ref(),
            setter: |this: &mut Arena, world| this.cfg.world = world,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "player",
            default: NodePath::from_str(Cfg::PLAYER),
            hint: PropertyHint::None,
            getter: |this: &Arena| this.cfg.player.new_ref(),
            setter: |this: &mut Arena, player| this.cfg.player = player,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "switch_file",
            default: Cfg::SWITCH.into(),
            hint: PropertyHint::None,
            getter: |this: &Arena| this.cfg.switch.new_ref(),
            setter: |this: &mut Arena, switch: GodotString| this.cfg.switch = switch,
            usage: default_usage,
        });
        builder.add_signal(Signal {
            name: Self::ARENA_READY.into(),
            args: &[],
        });
        builder.add_signal(Signal {
            name: Self::WAVE_NUMBER_CHANGED.into(),
            args: &[SignalArgument {
                name: "wave_num",
                default: Variant::from_u64(0),
                hint: PropertyHint::None,
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }
}

impl Arena {
    fn setup_next_wave(&mut self, mut owner: Node) {
        (|| {
            let cache = self.cache.as_ref()?;
            let mut instance = unsafe {
                cache.switch_scene
                    .lock().ok()
                    .tap_none(|| log::warn!("Could not load switch scene."))?
                    .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                    .tap_none(|| log::warn!("Could not instance switch scene."))?
                    .cast()
                    .tap_none(|| log::warn!("Could not cast instance switch to StaticBody2D."))?
            };
            Switch::call_instance_init(instance, unsafe { owner.get_path() }, "spawn_next_wave".into());
            unsafe {
                owner.add_child(Some(instance.to_node()), false);
                instance.set_global_position(gdnative::Vector2::new(512., 300.));
                let path = instance.get_path();
                self.spawned_switch_path = Some(path);
            }
            Some(())
        })();
    }
}

#[methods]
impl Arena {
    fn _init(_owner: Node) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&mut self, mut owner: Node) {
        // Load cache.
        self.spawn_sys.load_cache();
        self.cache = Cache::load_with(&self.cfg);
        // Report on state.
        log::info!("Hello from arena! Loaded with cfg: {:?}. Firing ready event.", self.cfg);
        self.setup_next_wave(owner);
        unsafe {
            owner.emit_signal(Self::ARENA_READY.into(), &[]);
            owner.emit_signal(
                Self::WAVE_NUMBER_CHANGED.into(),
                &[Variant::from_u64(self.wave.as_ref().map_or(0, |wave| wave.num()))],
            );
        }
    }

    #[export]
    fn _process(&mut self, _owner: Node, _delta: f64) {
    }

    #[export]
    fn remove_spawn(&mut self, owner: Node, _removing: Object) {
        self.spawn_count -= 1;
        if self.spawn_count == 0 {
            self.setup_next_wave(owner);
        }
    }

    #[export]
    fn spawn_next_wave(&mut self, mut owner: Node) {
        if let Some(path) = self.spawned_switch_path.take() {
            unsafe {
                owner
                    .get_node(path.new_ref())
                    .map(|mut n| n.queue_free());
            };
        }

        let wave = match self.wave.take() {
            None => Wave::initial(),
            Some(wave) => wave.successor(),
        };
        let world = unsafe { owner.get_node(self.cfg.world.new_ref()) };
        let spawns = self.spawn_sys.spawn_wave(&wave, world, self.cfg.arena_dim, self.cfg.arena_pos, self.cfg.player.new_ref());
        self.spawn_count = spawns.len() as u64;
        for mut spawn in spawns {
            let mut arr = VariantArray::new();
            arr.push(&Variant::from_object(&spawn));
            unsafe {
                spawn.connect(
                    "tree_exited".into(),
                    Some(owner.to_object()),
                    "remove_spawn".into(),
                    arr,
                    1,
                ).expect("No problems.");
            }
        }
        unsafe {
            owner.emit_signal(
                Self::WAVE_NUMBER_CHANGED.into(),
                &[Variant::from_u64(wave.num())],
            );
        }
        self.wave = Some(wave);
    }
}
