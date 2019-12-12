use gdnative::{
    Button,
    Control,
    CenterContainer,
    GodotString,
    HBoxContainer,
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
use std::fs::File;

use crate::{crafting::{RecipeVariant, Recipes}, systems::items::{Category, Inventory, Item, Stack}, util::{error, path_ops}};

lazy_static::lazy_static! {
    static ref base_pixel: Item = Item {
        category: Category::Raw,
        name: "base pixel".to_owned(),
        desc: "the source of all things".to_owned(),
        can_use: false,
    };
    static ref moderate_pixel: Item = Item {
        category: Category::Raw,
        name: "moderate pixel".to_owned(),
        desc: "7".to_owned(),
        can_use: false,
    };
    static ref advanced_pixel: Item = Item {
        category: Category::Raw,
        name: "advanced pixel".to_owned(),
        desc: "21".to_owned(),
        can_use: false,
    };
    static ref master_pixel: Item = Item {
        category: Category::Raw,
        name: "master pixel".to_owned(),
        desc: "42".to_owned(),
        can_use: false,
    };
    static ref generic_pixel: Item = Item {
        category: Category::Raw,
        name: "generic pixel".to_owned(),
        desc: "42n".to_owned(),
        can_use: false,
    };
    static ref cheap_health_potion: Item = Item {
        category: Category::Raw,
        name: "cheap health potion".to_owned(),
        desc: "restore up to 10 health".to_owned(),
        can_use: true,
    };
    static ref health_elixir: Item = Item {
        category: Category::Raw,
        name: "health elixir".to_owned(),
        desc: "increase max health by 10".to_owned(),
        can_use: true,
    };
    static ref recipes: Vec<Recipes> = vec![
        // pixel enhancement
        Recipes {
            input: maplit::hashmap! { (*base_pixel).clone() => 70u64 },
            output: vec![Stack { item: (*moderate_pixel).clone(), count: 1, }],
        },
        Recipes {
            input: maplit::hashmap! { (*moderate_pixel).clone() => 30u64 },
            output: vec![Stack { item: (*advanced_pixel).clone(), count: 1, }],
        },
        Recipes {
            input: maplit::hashmap! { (*advanced_pixel).clone() => 20u64 },
            output: vec![Stack { item: (*master_pixel).clone(), count: 1, }],
        },
        Recipes {
            input: maplit::hashmap! { (*master_pixel).clone() => 100u64 },
            output: vec![Stack { item: (*generic_pixel).clone(), count: 1, }],
        },
        // potions
        Recipes {
            input: maplit::hashmap! { (*base_pixel).clone() => 100u64 },
            output: vec![Stack { item: (*cheap_health_potion).clone(), count: 1, }],
        },
        Recipes {
            input: maplit::hashmap! { (*master_pixel).clone() => 100u64 },
            output: vec![Stack { item: (*health_elixir).clone(), count: 1, }],
        },
    ];
}

pub struct Crafting {
    marking_col: NodePath,
    in_col: NodePath,
    out_col: NodePath,
    player: NodePath,
    recipe_dir: String,
    listing: Vec<Recipes>,
}

impl Crafting {
    const MARKING_PATH: &'static str = "";
    const IN_PATH: &'static str = "";
    const OUT_PATH: &'static str = "";
    const PLAYER_PATH: &'static str = "";
    const RECIPE_DIR: &'static str = "";
}

impl Default for Crafting {
    fn default() -> Self {
        Self {
            marking_col: Self::MARKING_PATH.into(),
            in_col: Self::IN_PATH.into(),
            out_col: Self::OUT_PATH.into(),
            player: Self::PLAYER_PATH.into(),
            recipe_dir: Self::RECIPE_DIR.to_owned(),
            listing: Default::default(),
        }
    }
}

impl NativeClass for Crafting {
    type Base = CenterContainer;
    type UserData = MutexData<Crafting>;

    fn class_name() -> &'static str {
        "Crafting"
    }

    fn init(_owner: Self::Base) -> Self {
        Default::default()
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        builder.add_property(Property {
            name: "recipe_dir",
            default: Self::RECIPE_DIR.into(),
            hint: PropertyHint::None,
            getter: |this: &Self| this.recipe_dir.clone().into(),
            setter: |this: &mut Self, dir: GodotString| this.recipe_dir = dir.to_string(),
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "marking_path",
            default: NodePath::from_str(Self::MARKING_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.marking_col.new_ref(),
            setter: |this: &mut Self, path| this.marking_col = path,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "in_path",
            default: NodePath::from_str(Self::IN_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.in_col.new_ref(),
            setter: |this: &mut Self, path| this.in_col = path,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "out_path",
            default: NodePath::from_str(Self::OUT_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.out_col.new_ref(),
            setter: |this: &mut Self, path| this.out_col = path,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "player_path",
            default: NodePath::from_str(Self::PLAYER_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.player.new_ref(),
            setter: |this: &mut Self, path| this.player = path,
            usage: PropertyUsage::DEFAULT,
        });
    }
}

#[methods]
impl Crafting {
    #[export]
    fn _ready(&mut self, _owner: CenterContainer) {
        self.recipe_dir = path_ops::abs_asset(self.recipe_dir.clone());
        log::info!("Attempting to laod file from {:?}.", self.recipe_dir);
        self.listing = File::open(&self.recipe_dir)
            .map_err(error::JsonIOError::IO)
            .and_then(|f| json::from_reader(f).map_err(error::JsonIOError::Json))
            .tap_err(|e| log::error!("Could not find recipe due to {:?}!", e))
            .unwrap_or_else(|_| recipes.clone());
    }

    fn recipe_columns(&self, owner: &CenterContainer) -> Option<(VBoxContainer, VBoxContainer, VBoxContainer)> {
        unsafe {
            Some((
                owner.get_node(self.marking_col.new_ref())?.cast()?,
                owner.get_node(self.in_col.new_ref())?.cast()?,
                owner.get_node(self.out_col.new_ref())?.cast()?,
            ))
        }
    }

    fn create_entry(recipe: &Recipes, inv: &Inventory) -> (Button, Control, Control) {
        let mut mark = Button::new();
        let mut ins = HBoxContainer::new();
        let mut outs = HBoxContainer::new();
        unsafe {
            let (avail, mark_text) = if recipe.input.iter().any(|(item, count)| inv.count_items(item) >= *count) {
                (true, "Available")
            } else {
                (false, "Cannot make")
            };
            mark.set_disabled(!avail);
            mark.set_text(mark_text.into());
            for input in recipe.input.iter() {
                let mut name = Label::new();
                let mut num = Label::new();
                name.set_text(input.0.name.clone().into());
                num.set_text(input.1.to_string().into());
                ins.add_child(Some(name.to_node()), false);
                ins.add_child(Some(num.to_node()), false);
            }
            for output in recipe.output.iter() {
                let mut name = Label::new();
                let mut num = Label::new();
                name.set_text(output.item.name.clone().into());
                num.set_text(output.count.to_string().into());
                outs.add_child(Some(name.to_node()), false);
                outs.add_child(Some(num.to_node()), false);
            }
            mark.set_custom_minimum_size(gdnative::Vector2::new(0., 50.));
            ins.set_custom_minimum_size(gdnative::Vector2::new(0., 50.));
            outs.set_custom_minimum_size(gdnative::Vector2::new(0., 50.));
            (
                mark,
                ins.to_control(),
                outs.to_control(),
            )
        }
    }

    #[export]
    pub fn render_recipes(&self, owner: CenterContainer) {
        log::info!("Rendering recipes: {:?}", self.listing);
        unsafe {
            if let Some(node) = owner.get_node(self.player.new_ref()).and_then(|n| n.cast()) {
                if let Some(player) = Instance::<crate::entity::Player>::try_from_base(node) {
                    player.map_mut(move |player, base| {
                        if let Some(mut recipe_columns) = self.recipe_columns(&owner) {
                            while let Some(child) = recipe_columns.0.get_child(0) {
                                recipe_columns.0.remove_child(Some(child));
                            }
                            while let Some(child) = recipe_columns.1.get_child(0) {
                                recipe_columns.1.remove_child(Some(child));
                            }
                            while let Some(child) = recipe_columns.2.get_child(0) {
                                recipe_columns.2.remove_child(Some(child));
                            }
                            self.listing
                                .iter()
                                .map(|recipe| (recipe, Self::create_entry(recipe, &player.inventory)))
                                .for_each(|(recipe, (mut mark, ins, outs))| {
                                    let mut arr = VariantArray::new();
                                    arr.push(&RecipeVariant::from(recipe.clone()).to_variant());
                                    mark.connect("button_up".into(), Some(base.to_object()), "craft_recipe".into(), arr, 0);
                                    mark.connect("button_up".into(), Some(owner.to_object()), "render_recipes".into(), VariantArray::new(), 0);
                                    recipe_columns.0.add_child(Some(mark.to_node()), false);
                                    recipe_columns.1.add_child(Some(ins.to_node()), false);
                                    recipe_columns.2.add_child(Some(outs.to_node()), false);
                                });
                        }
                    }).tap_err(|e| log::error!("Could not find player due to {:?}!", e));
                }
            }
        }
    }
}
