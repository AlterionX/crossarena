use gdnative::{init::PropertyHint, GodotString, Node};

const ENEMY: &'static str = "enemy";
const PLAYER: &'static str = "player";
const SWITCH: &'static str = "player";
const PROJECTILE: &'static str = "projectile";

const PLAYER_MASK: i64 = 1 << 0;
const PROJECTILE_MASK: i64 = 1 << 2;
const ENEMY_MASK: i64 = 1 << 1;
const SWITCH_MASK: i64 = 1 << 3;

lazy_static::lazy_static! {
    static ref ENEMY_GS: GodotString = ENEMY.into();
    static ref PLAYER_GS: GodotString = PLAYER.into();
    static ref PROJECTILE_GS: GodotString = PROJECTILE.into();
    static ref SWITCH_GS: GodotString = SWITCH.into();
}

pub enum Group {
    Enemy,
    Player,
    Switch,
    Projectile,
}

impl Group {
    pub const ALL_GROUPS: &'static[Self] = &[Self::Player, Self::Enemy, Self::Projectile, Self::Switch];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Enemy => ENEMY,
            Self::Player => PLAYER,
            Self::Projectile => PROJECTILE,
            Self::Switch => &SWITCH,
        }
    }

    pub fn godot_name(&self) -> &'static GodotString {
        match self {
            Self::Enemy => &ENEMY_GS,
            Self::Player => &PLAYER_GS,
            Self::Projectile => &PROJECTILE_GS,
            Self::Switch => &SWITCH_GS,
        }
    }

    pub fn physics_mask(&self) -> i64 {
        match self {
            Self::Enemy => ENEMY_MASK,
            Self::Player => PLAYER_MASK,
            Self::Projectile => PROJECTILE_MASK,
            Self::Switch => SWITCH_MASK,
        }
    }

    pub fn collected_physics_mask(groups: &[Self]) -> i64 {
        groups.iter().fold(0, |flags, group| flags | group.physics_mask())
    }

    pub fn add_node(&self, mut node: Node) {
        unsafe {
            node.add_to_group(
                self.godot_name().new_ref(),
                false,
            )
        }
    }

    pub fn has_node(&self, node: Node) -> bool {
        unsafe {
            node.is_in_group(self.godot_name().new_ref())
        }
    }

    pub fn full_property_hint() -> PropertyHint<'static> {
        PropertyHint::Enum {
            values: &[ENEMY, PLAYER, PROJECTILE],
        }
    }
}

impl From<&Group> for &'static GodotString {
    fn from(g: &Group) -> Self {
        g.godot_name()
    }
}

impl From<Group> for &'static GodotString {
    fn from(g: Group) -> Self {
        g.godot_name()
    }
}

impl From<&Group> for GodotString {
    fn from(g: &Group) -> Self {
        g.godot_name().new_ref()
    }
}

impl From<Group> for GodotString {
    fn from(g: Group) -> Self {
        g.godot_name().new_ref()
    }
}

impl From<&Group> for &'static str {
    fn from(g: &Group) -> Self {
        g.name()
    }
}

impl From<Group> for &'static str {
    fn from(g: Group) -> Self {
        g.name()
    }
}

impl From<&Group> for String {
    fn from(g: &Group) -> Self {
        g.name().to_owned()
    }
}

impl From<Group> for String {
    fn from(g: Group) -> Self {
        g.name().to_owned()
    }
}
