use gdnative::{Node, NodePath};
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
