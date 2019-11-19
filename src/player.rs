use nalgebra as na;
use gdnative::{
    self as godot,
    InputEvent,
    InputEventMouseButton,
    InputEventMouseMotion,
    InputEventKey,
    KinematicBody2D,
};
use std::time::Duration;
use crate::{
    direction::Direction,
    conv,
    player::{
        aim::{System as AimSys, Cfg as AimCfg},
        dash::{System as DashSys, Cfg as DashCfg},
        melee::{System as MeleeSys, Cfg as MeleeCfg},
    },
    health::{System as HealthSys, Cfg as HealthCfg},
    inventory::Inventory,
    systems::EditorCfg,
};

mod dash;
mod aim;
mod melee;

pub struct Player {
    // Basic movement
    facing_dir: Direction,
    base_speed: f64,
    accel_time: f64,
    remaining_accel: f64,
    melee_radius: f64,

    dash: DashSys,
    aim: AimSys,
    melee: MeleeSys,
    health: HealthSys,

    // inventory
    inventory: Inventory,
}

impl Player {
    const DEFAULT_BASE_SPEED: f64 = 140.;
    const DEFAULT_ACCEL_TIME: f64 = 1.;
    const MELEE_RADIUS: f64 = 30.;
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // Basic movement
            facing_dir: Default::default(),
            base_speed: Self::DEFAULT_BASE_SPEED,
            accel_time: Self::DEFAULT_ACCEL_TIME,
            remaining_accel: Self::DEFAULT_ACCEL_TIME,
            melee_radius: Self::MELEE_RADIUS,

            dash: Default::default(),
            aim: Default::default(),
            melee: Default::default(),
            health: Default::default(),

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
        DashCfg::register_properties(builder, |this| &this.dash.cfg, |this| &mut this.dash.cfg);
        AimCfg::register_properties(builder, |this| &this.aim.cfg, |this| &mut this.aim.cfg);
        MeleeCfg::register_properties(builder, |this| &this.melee.cfg, |this| &mut this.melee.cfg);
        HealthCfg::register_properties(builder, |this| &this.health.cfg, |this| &mut this.health.cfg);
    }
}

impl Player {
    fn handle_mouse_button(&mut self, event: InputEventMouseButton, owner: KinematicBody2D) {
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
            let own_pos = unsafe { owner.get_global_position() };
            let mouse_pos = event.to_input_event_mouse().get_global_position();
            let dist_from_sprite = (own_pos - mouse_pos).length() as f64;
            match button {
                // Attacks
                BUTTON_L => if self.dash.is_dashing() {
                    // Do nothing
                } else if dist_from_sprite > self.melee_radius {
                    self.melee.reset();
                    self.aim.aim_at(conv::g_to_na64(mouse_pos))
                } else {
                    self.melee.attack(unsafe { owner.to_node_2d() }, self.facing_dir);
                },
                // Dashing
                // Overrides any other action.
                BUTTON_R => {
                    // TODO reset any other state.
                    self.aim.reset();
                    self.melee.reset();
                    self.health.set_invincibility(self.dash.invincibility());
                    self.dash.dash(self.facing_dir);
                },
                _ => (),
            }
        } else { // released
            match button {
                BUTTON_L => log::info!("Released L mouse button."),
                BUTTON_R => log::info!("Released R mouse button."),
                _ => (),
            }
            let own_pos = conv::g_to_na64(unsafe { owner.get_global_position() });
            match button {
                // Begin aiming
                BUTTON_L => if self.dash.is_dashing() {
                    // Do nothing
                } else if self.aim.is_aiming() {
                    self.aim.attack(own_pos, unsafe { owner.to_node() }, self.calc_projectile_dmg());
                },
                _ => (),
            }
        }
    }
    fn handle_mouse_motion(&mut self, event: InputEventMouseMotion) {
        if self.aim.is_aiming() {
            let mouse_pos = conv::g_to_na64(event.to_input_event_mouse().get_global_position());
            self.aim.aim_at(mouse_pos);
        }
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
            KEY_0..=KEY_9 | KEY_A..=KEY_Z => log::trace!("Alphanumeric key pressed: {:?}", scancode as u8 as char),
            _ => log::trace!("Key with scancode {:?} pressed.", scancode),
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

    fn calc_projectile_dmg(&self) -> f64 {
        self.aim.calc_dmg()
    }
    fn calc_vel(&mut self, delta: Duration) -> na::Vector2<f64> {
        // TODO Change `to_na_vec` to `into` when able to.
        // TODO Add an "acceleration" time.
        self.dash.calc_vel(delta)
            .or_else(|| self.aim.calc_vel(self.facing_dir))
            .or_else(|| self.melee.calc_vel(self.facing_dir))
            .unwrap_or_else(|| self.facing_dir.to_na_vec() * self.base_speed)
    }
}

#[methods]
impl Player {
    fn _init() -> Self {
        Player::default()
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: KinematicBody2D) {
        self.aim.load_cache();
        self.melee.load_cache();
        self.health.init();
        log::info!("Hello from the player.");
    }

    #[export]
    fn _process(&mut self, _owner: KinematicBody2D, delta: f64) {
        let delta = Duration::from_secs_f64(delta);
        self.melee.process(delta);
        self.health.process(delta);
        self.aim.narrow_aim(delta);
    }

    #[export]
    unsafe fn _physics_process(&mut self, mut owner: KinematicBody2D, delta: f64) {
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
    unsafe fn _input(&mut self, owner: KinematicBody2D, event: Option<InputEvent>) {
        if let Some(mouse_motion_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_mouse_motion(mouse_motion_event);
        } else if let Some(key_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_key(key_event);
        } else if let Some(mouse_button_event) = event.as_ref().and_then(|e| e.cast()) {
            self.handle_mouse_button(mouse_button_event, owner);
        }
    }

    #[export]
    fn emit_init_signals(&self, owner: KinematicBody2D) {
        let mut owner = unsafe { owner.to_object() };
        self.health.broadcast_max_hp(&mut owner);
        self.health.broadcast_hp(&mut owner);
    }
}
