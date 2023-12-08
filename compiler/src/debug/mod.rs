#[cfg(feature = "debug")]
pub mod display;
pub mod macros;
#[cfg(feature = "debug")]
pub mod time;

#[cfg(feature = "debug")]
use {clap::ValueEnum, miette::miette, once_cell::sync::OnceCell};

#[cfg(feature = "debug")]
#[derive(Debug)]
pub struct DebugArgs {
    pub time: bool,
    pub display: Option<Pass>,
}

#[cfg(feature = "debug")]
impl DebugArgs {
    pub fn set(time: bool, display: Option<Pass>) -> miette::Result<()> {
        DEBUG_ARGS
            .set(DebugArgs { time, display })
            .map_err(|_| miette!("Failed to set up `DebugArgs`."))
    }
}

#[cfg(feature = "debug")]
pub static DEBUG_ARGS: OnceCell<DebugArgs> = OnceCell::new();

#[cfg(feature = "debug")]
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Pass {
    Parse,
    Validate,
    Reveal,
    Atomize,
    Explicate,
    Select,
    Assign,
}
