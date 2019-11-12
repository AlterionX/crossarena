use nalgebra as na;
use gdnative::{
    self as godot,
    GodotString,
    InputEvent,
    InputEventMouseButton,
    InputEventMouseMotion,
    InputEventKey,
    init::{Property, PropertyHint, PropertyUsage}
};
use std::time::{Duration, Instant};
use crate::{
    direction::Direction,
    inventory::Inventory,
    conv,
};

#[derive(Debug)]
struct DashCfg {
    duration: Duration,
    cooldown: Duration,
    slowdown: f64,
    speed: f64,
    chain: u8,
}

impl DashCfg {
    const SPEED: f64 = 1000.;
    const CHAIN: u8 = 3;
    const DURATION: Duration = Duration::from_millis(200);
    const COOLDOWN: Duration = Duration::from_millis(50);
    const SLOWDOWN: f64 = 0.5;
}

impl Default for DashCfg {
    fn default() -> Self {
        Self {
            duration: Self::DURATION,
            cooldown: Self::COOLDOWN,
            slowdown: Self::SLOWDOWN,
            speed: Self::SPEED,
            chain: Self::CHAIN,
        }
    }
}

struct DashData {
    count: u8,
    since_start: Duration,
    dir: Direction,
}

impl DashData {
    fn first_dash(d: Direction) -> Self {
        Self {
            count: 0,
            since_start: Duration::from_secs(0),
            dir: d,
        }
    }
    fn chained_dash(&self, d: Direction) -> Self {
        Self {
            count: self.count + 1,
            since_start: Duration::from_secs(0),
            dir: d,
        }
    }
}

struct AimCfg {
    max_aim_time: Duration,
    min_aim_time: Duration,
}

impl AimCfg {
    const MAX_AIM: Duration = Duration::from_millis(300);
    const MIN_AIM: Duration = Duration::from_millis(100);
}

impl Default for AimCfg {
    fn default() -> Self {
        Self {
            max_aim_time: Self::MAX_AIM,
            min_aim_time: Self::MIN_AIM,
        }
    }
}

pub struct Player {
    // Basic movement
    facing_dir: Direction,
    aiming_loc: Option<godot::Vector2>,
    base_speed: f64,
    accel_time: f64,
    remaining_accel: f64,

    // Dashing
    dash: DashCfg,
    dashing: Option<DashData>,

    // Aiming
    aim: AimCfg,
    aiming: Option<Instant>,

    // inventory
    inventory: Inventory,
}

impl Player {
    const DEFAULT_BASE_SPEED: f64 = 250.;
    const DEFAULT_ACCEL_TIME: f64 = 1.;
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // Basic movement
            facing_dir: Default::default(),
            aiming_loc: Default::default(),
            base_speed: Self::DEFAULT_BASE_SPEED,
            accel_time: Self::DEFAULT_ACCEL_TIME,
            remaining_accel: Self::DEFAULT_ACCEL_TIME,

            // Dashing
            dash: Default::default(),
            dashing: Default::default(),

            // Aiming
            aim: Default::default(),
            aiming: Default::default(),

            // Inventory
            inventory: Default::default(),
        }
    }
}

impl godot::NativeClass for Player {
    type Base = godot::KinematicBody2D;
    type UserData = godot::user_data::MutexData<Player>;

    fn class_name() -> &'static str {
        "Player"
    }

    fn init(_owner: Self::Base) -> Self {
        Self::_init()
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        builder.add_property(Property {
            name: "base/dash_speed",
            default: 0.05,
            hint: PropertyHint::Range {
                range: 0.05..1.0,
                step: 0.01,
                slider: true,
            },
            getter: |this: &Player| this.dash.speed,
            setter: |this: &mut Player, v| this.dash.speed = v,
            usage: PropertyUsage::DEFAULT,
        });

        builder.add_property(Property {
            name: "test/test_enum",
            default: GodotString::from_str("Hello"),
            hint: PropertyHint::Enum {
                values: &["Hello", "World", "Testing"],
            },
            getter: |_: &Player| GodotString::from_str("Hello"),
            setter: (),
            usage: PropertyUsage::DEFAULT,
        });

        builder.add_property(Property {
            name: "test/test_flags",
            default: 0,
            hint: PropertyHint::Flags {
                values: &["A", "B", "C", "D"],
            },
            getter: |_: &Player| 0,
            setter: (),
            usage: PropertyUsage::DEFAULT,
        });
    }
}

impl Player {
    fn handle_mouse_button(&mut self, event: InputEventMouseButton, owner: godot::KinematicBody2D) {
        const BUTTON_L: i64 = 1;
        const BUTTON_R: i64 = 2;

        let button = event.get_button_index();
        let pressed = event.is_pressed();

        if pressed {
            match button {
                BUTTON_L => log::info!("Pressed L mouse button."),
                BUTTON_R => log::info!("Pressed R mouse button."),
                _ => (),
            }
            match button {
                BUTTON_L => (), // attack
                BUTTON_R => if let Some(dashing) = &mut self.dashing {
                    // Already dashing, so change direction and advance dash count.
                    if dashing.count < self.dash.chain {
                        self.dashing = Some(dashing.chained_dash(self.facing_dir));
                    } else {
                        dashing.count = self.dash.chain;
                        dashing.since_start = Duration::from_secs(0);
                    }
                } else {
                    // Not yet dashing. Begin to dash.
                    self.dashing = Some(DashData::first_dash(self.facing_dir));
                },
                _ => (),
            }
        }
    }
    fn handle_mouse_motion(&mut self, event: InputEventMouseMotion) {
    }
    fn handle_key(&mut self, event: InputEventKey) {
        const KEY_A: i64 = b'A' as i64;
        const KEY_D: i64 = b'D' as i64;
        const KEY_S: i64 = b'S' as i64;
        const KEY_W: i64 = b'W' as i64;
        const KEY_Z: i64 = b'Z' as i64;
        const KEY_0: i64 = b'0' as i64;
        const KEY_9: i64 = b'9' as i64;

        if event.is_echo() { return; }

        let scancode = event.get_scancode();

        match scancode {
            KEY_0..=KEY_9 | KEY_A..=KEY_Z => log::info!("Alphanumeric key pressed: {:?}", scancode as u8 as char),
            _ => log::info!("Key with scancode {:?} pressed.", scancode),
        }

        self.facing_dir = match (scancode, event.is_pressed()) {
            (KEY_W, true) | (KEY_S, false) => self.facing_dir.shift_up(),
            (KEY_A, true) | (KEY_D, false) => self.facing_dir.shift_left(),
            (KEY_S, true) | (KEY_W, false) => self.facing_dir.shift_down(),
            (KEY_D, true) | (KEY_A, false) => self.facing_dir.shift_right(),
            _ => self.facing_dir,
        };
        if let KEY_W | KEY_A | KEY_S | KEY_D = scancode {
            log::info!(
                "Direction changed! Key {} was {}. Direction is now {:?}.",
                scancode as u8 as char,
                if event.is_pressed() { "pressed" } else { "released" },
                self.facing_dir,
            );
        }
    }

    fn calc_vel(&mut self, delta: Duration) -> na::Vector2<f64> {
        // TODO Change `to_na_vec` to `into` when able to.
        // TODO Add an "acceleration" time.
        if let Some(dashing) = &mut self.dashing {
            dashing.since_start += delta;
            if self.dash.chain <= dashing.count || dashing.since_start > self.dash.duration {
                if dashing.since_start > self.dash.duration + self.dash.cooldown {
                    self.dashing = None;
                }
                self.facing_dir.to_na_vec() * self.base_speed * self.dash.slowdown
            } else {
                dashing.dir.to_na_vec() * self.dash.speed
            }
        } else {
            self.facing_dir.to_na_vec() * self.base_speed
        }
    }
}

#[godot::methods]
impl Player {
    fn _init() -> Self {
        Player::default()
    }

    #[export]
    unsafe fn _ready(&mut self, mut owner: godot::KinematicBody2D) {
        owner.set_physics_process(true);
        log::info!("Hello from the player.");
    }

    #[export]
    unsafe fn _physics_process(&mut self, mut owner: godot::KinematicBody2D, delta: f64) {
        let delta = Duration::from_secs_f64(delta);
        owner.move_and_slide(
            conv::na64_to_g(-self.calc_vel(delta)),
            godot::Vector2::zero(),
            false,
            3,
            0.4333,
            true,
        );
    }

    #[export]
    unsafe fn _input(&mut self, owner: godot::KinematicBody2D, event: Option<InputEvent>) {
        if let Some(mouse_motion_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_mouse_motion(mouse_motion_event);
        } else if let Some(key_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_key(key_event);
        } else if let Some(mouse_button_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_mouse_button(mouse_button_event, owner);
        }
    }
}

#[derive(godot::NativeClass)]
#[inherit(godot::KinematicBody2D)]
pub struct Projectile {
    dir: godot::Vector2,
    bounces: u8,
}

#[gdnative::methods]
impl Projectile {
    fn _init(_owner: gdnative::KinematicBody2D) -> Self {
        Projectile {
            dir: godot::Vector2::new(0., 0.),
            bounces: 0,
        }
    }

    #[export]
    fn _ready(&self, _owner: gdnative::KinematicBody2D) {
        godot_print!("hello, world.")
    }

    #[export]
    fn initialize_pos_and_dir(
        &mut self,
        mut owner: gdnative::KinematicBody2D,
        pos: godot::Vector2,
        dir: godot::Vector2,
        bounces: u8,
    ) {
        unsafe { owner.set_position(pos); }
        self.dir = dir;
        self.bounces = bounces;
    }
}
