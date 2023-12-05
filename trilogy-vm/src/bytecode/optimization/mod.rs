use super::chunk::Line;

mod remove_noops;

use remove_noops::*;

pub(super) fn optimize(
    lines: Vec<Line>,
    _entrypoint: usize,
    _force_reachable: &[String],
) -> Vec<Line> {
    remove_noops(lines)
}
