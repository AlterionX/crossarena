use gdnative::{Node, NodePath, ProjectSettings};
use tap::TapOptionOps;

pub fn to_abs_if_exist(target: NodePath, root: &Node) -> Option<NodePath> {
    unsafe {
        root
            .get_node(target.new_ref())
            .map(|target| target.get_path())
            .tap_none(|| log::warn!(
                    "Provided target path {} to node {} cannot be found!",
                    target.to_string(),
                    root.get_name().to_string(),
                )
            )
    }
}

// Convert to system path.
pub fn abs_asset(path: String) -> String {
    let project_settings = ProjectSettings::godot_singleton();
    // Convert to system path.
    project_settings.globalize_path(path.into()).to_string()
}
