use gdnative::{
    init::{Property, PropertyHint, PropertyUsage, Signal,},
    NodePath,
    CanvasItem,
    Control,
    Instance,
    InputEvent,
    InputEventKey,
};

use tap::TapResultOps;

mod hud;
pub use hud::HUD;
mod end;
pub use end::End;
mod start;
pub use start::Start;
mod inventory;
pub use inventory::Inventory;
mod crafting;
pub use crafting::Crafting;

#[derive(Debug)]
struct Cfg {
    hud_path: NodePath,
    crafting: NodePath,
    inventory: NodePath,
}

impl Cfg {
    const HUD_PATH: &'static str = "HUD";
    const CRAFTING_PATH: &'static str = "HUD";
    const INVENTORY_PATH: &'static str = "HUD";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            hud_path: NodePath::from_str(Self::HUD_PATH),
            crafting: NodePath::from_str(Self::CRAFTING_PATH),
            inventory: NodePath::from_str(Self::INVENTORY_PATH),
        }
    }
}

#[derive(Default)]
pub struct UI {
    cfg: Cfg,
}

impl UI {
    const BEGIN_WAVE: &'static str = "trigger_wave";
}

impl godot::NativeClass for UI {
    type Base = Control;
    type UserData = godot::user_data::MutexData<UI>;

    fn class_name() -> &'static str {
        "UI"
    }

    fn init(owner: Self::Base) -> Self {
        Default::default()
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        let default_usage = PropertyUsage::SCRIPT_VARIABLE | PropertyUsage::STORAGE | PropertyUsage::EDITOR;
        builder.add_property(Property {
            name: "hud",
            default: NodePath::from_str(Cfg::HUD_PATH),
            hint: PropertyHint::None,
            getter: |this: &UI| this.cfg.hud_path.new_ref(),
            setter: |this: &mut UI, world| this.cfg.hud_path = world,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "crafting",
            default: NodePath::from_str(Cfg::CRAFTING_PATH),
            hint: PropertyHint::None,
            getter: |this: &UI| this.cfg.crafting.new_ref(),
            setter: |this: &mut UI, world| this.cfg.crafting = world,
            usage: default_usage,
        });
        builder.add_property(Property {
            name: "inventory",
            default: NodePath::from_str(Cfg::INVENTORY_PATH),
            hint: PropertyHint::None,
            getter: |this: &UI| this.cfg.inventory.new_ref(),
            setter: |this: &mut UI, world| this.cfg.inventory = world,
            usage: default_usage,
        });
        builder.add_signal(Signal {
            name: "resume".into(),
            args: &[],
        })
    }
}

#[methods]
impl UI {
    #[export]
    fn _ready(&mut self, _owner: Control) {
        log::info!("Hello from the hud!");
    }

    #[export]
    fn _input(&self, owner: Control, input: InputEvent) {
        if let Some(key) = input.cast::<InputEventKey>() {
            if key.is_pressed()
                && key.get_scancode() == gdnative::GlobalConstants::KEY_ESCAPE
            {
                if unsafe {
                    owner.get_tree().map(|tree| tree.is_paused())
                }.unwrap_or(false) {
                    self.resume_game(owner);
                } else {
                    self.to_inventory(owner);
                }
            }
        }
    }

    pub fn to_forge(&self, owner: Control) {
        unsafe {
            if let Some(mut tree) = owner.to_node().get_tree() {
                if let Some(forge_ui) = owner.get_node(self.cfg.crafting.new_ref()) {
                    if let Some(mut ui) = forge_ui.cast::<CanvasItem>() {
                        if let Some(ui) = ui.cast() {
                            if let Some(instance) = Instance::<crate::ui::Crafting>::try_from_base(ui) {
                                instance.map_mut(|ui, base| {
                                    ui.render_recipes(base);
                                })
                                .tap_err(|e| log::error!("Could not invoke `render_recipes` due to {:?}.", e));
                            }
                        }
                        ui.set_visible(true);
                    }
                }
                tree.set_pause(true);
            }
        }
    }

    fn to_inventory(&self, owner: Control) {
        unsafe {
            if let Some(mut tree) = owner.to_node().get_tree() {
                if let Some(inventory_ui) = owner.get_node(self.cfg.inventory.new_ref()) {
                    if let Some(mut ui) = inventory_ui.cast::<CanvasItem>() {
                        if let Some(ui) = ui.cast() {
                            if let Some(instance) = Instance::<crate::ui::Inventory>::try_from_base(ui) {
                                instance.map_mut(|ui, base| {
                                    ui.render_inventory(base);
                                })
                                .tap_err(|e| log::error!("Could not invoke `render_inventory` due to {:?}.", e));
                            }
                        }
                        ui.set_visible(true);
                    }
                }
                tree.set_pause(true);
            }
        }
    }

    fn resume_game(&self, mut owner: Control) {
        unsafe {
            if let Some(mut tree) = owner.to_node().get_tree() {
                tree.set_pause(false);
                if let Some(forge_ui) = owner.get_node(self.cfg.crafting.new_ref()) {
                    if let Some(mut ui) = forge_ui.cast::<CanvasItem>() {
                        ui.set_visible(false);
                    }
                }
                if let Some(inventory_ui) = owner.get_node(self.cfg.inventory.new_ref()) {
                    if let Some(mut ui) = inventory_ui.cast::<CanvasItem>() {
                        ui.set_visible(false);
                    }
                }
                owner.emit_signal("resume".into(), &[]);
            }
        }
    }
}
