//! What every pklith-dev test shares.

use std::path::Path;
use std::process::Command;

/// Variables git exports to hooks (root V20). The gate runs these tests
/// from a hook; a child that inherits them acts on the developer's
/// repository whatever directory it runs in, so a fixture's `git commit`
/// lands on their branch.
const HOOK_VARS: [&str; 3] = ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"];

/// `program` run in `dir`, with [`HOOK_VARS`] removed, as
/// `pklith::proc::command` does for the product crate.
pub fn command(program: &str, dir: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir);
    for var in HOOK_VARS {
        cmd.env_remove(var);
    }
    cmd
}
