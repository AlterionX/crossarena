use gdnative::{
    Button,
    CenterContainer,
    Control,
    Container,
    GodotString,
    init::{ClassBuilder, Property, PropertyHint, PropertyUsage},
    Instance,
    Label,
    NativeClass,
    NodePath,
    ToVariant,
    user_data::MutexData,
    VariantArray,
    VBoxContainer,
};
use tap::TapResultOps;

use crate::systems::items;

pub struct Inventory {
    item_grid_path: NodePath,
    player: NodePath,
}

impl Inventory {
    const ITEM_GRID_PATH: &'static str = "";
    const PLAYER_PATH: &'static str = "";
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            item_grid_path: Self::ITEM_GRID_PATH.into(),
            player: Self::PLAYER_PATH.into(),
        }
    }
}

impl NativeClass for Inventory {
    type Base = CenterContainer;
    type UserData = MutexData<Inventory>;

    fn class_name() -> &'static str {
        "Inventory"
    }

    fn init(_owner: Self::Base) -> Self {
        Default::default()
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        builder.add_property(Property {
            name: "item_grid",
            default: Self::ITEM_GRID_PATH.into(),
            hint: PropertyHint::None,
            getter: |this: &Self| this.item_grid_path.new_ref(),
            setter: |this: &mut Self, path| this.item_grid_path = path,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "player",
            default: Self::PLAYER_PATH.into(),
            hint: PropertyHint::None,
            getter: |this: &Self| this.player.new_ref(),
            setter: |this: &mut Self, path| this.player = path,
            usage: PropertyUsage::DEFAULT,
        });
    }
}

impl Inventory {
    fn create_item_box(stack: &items::Stack) -> (Button, CenterContainer) {
        let mut root = CenterContainer::new();
        let mut content = VBoxContainer::new();
        let mut name_label = Label::new();
        let mut desc_label = Label::new();
        let mut quant_label = Label::new();
        let mut use_button = Button::new();
        unsafe {
            name_label.set_text(stack.item.name.clone().into());
            desc_label.set_text(stack.item.desc.clone().into());
            quant_label.set_text(stack.count.to_string().into());
            if stack.item.can_use {
                use_button.set_disabled(false);
                use_button.set_text("Use".into());
            } else {
                use_button.set_disabled(true);
                use_button.set_text("Cannot be used directly.".into());
            }
            content.add_child(Some(name_label.to_node()), false);
            content.add_child(Some(desc_label.to_node()), false);
            content.add_child(Some(quant_label.to_node()), false);
            content.add_child(Some(use_button.to_node()), false);
            root.add_child(Some(content.to_node()), false);
        }
        (use_button, root)
    }
}

#[methods]
impl Inventory {
    #[export]
    pub fn render_inventory(&self, owner: CenterContainer) {
        unsafe {
            if let Some(player) = owner.get_node(self.player.new_ref()).and_then(|n| n.cast()) {
                if let Some(instance) = Instance::<crate::entity::Player>::try_from_base(player) {
                    instance.map_mut(|player, base| {
                        if let Some(mut item_grid) = owner.get_node(self.item_grid_path.new_ref()) {
                            while let Some(child) = item_grid.get_child(0) {
                                item_grid.remove_child(Some(child));
                            }
                            player.inventory
                                .stacks()
                                .map(|stack| (stack, Self::create_item_box(stack)))
                                .for_each(|(stack, (mut button, item))| {
                                    let mut arr = VariantArray::new();
                                    arr.push(&stack.item.to_variant());
                                    button.connect("button_up".into(), Some(base.to_object()), "use_item".into(), arr, 0)
                                        .tap_err(|e| log::error!("Could not connect item usage signal due to {:?}.", e));
                                    button.connect("button_up".into(), Some(owner.to_object()), "render_inventory".into(), VariantArray::new(), 0)
                                        .tap_err(|e| log::error!("Could not connect item usage signal due to {:?}.", e));
                                    item_grid.add_child(Some(item.to_node()), false)
                                });
                        }
                    })
                    .tap_err(|e| log::error!("Could not render inventory due to {:?}.", e));
                }
            }
        }
    }
}
