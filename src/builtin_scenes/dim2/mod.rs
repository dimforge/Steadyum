use crate::builtin_scenes::BuiltinScene;
use std::cmp::Ordering;

mod pyramids2;

pub fn builders() -> Vec<(&'static str, fn() -> BuiltinScene)> {
    let mut builders: Vec<(&'static str, fn() -> BuiltinScene)> =
        vec![("Pyramids (heavy)", pyramids2::init_world)];

    // Lexicographic sort, with stress tests moved at the end of the list.
    builders.sort_by(|a, b| match (a.0.starts_with("("), b.0.starts_with("(")) {
        (true, true) | (false, false) => a.0.cmp(b.0),
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
    });

    builders
}
