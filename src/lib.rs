use emacs::{Env, Result};

pub mod dynmod;

emacs::plugin_is_GPL_compatible!();

#[emacs::module(mod_in_name = false)]
fn init(_: &Env) -> Result<()> {
    Ok(())
}
