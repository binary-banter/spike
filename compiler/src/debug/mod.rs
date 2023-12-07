pub mod display;
pub mod time;

use clap::ValueEnum;
use miette::miette;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct DebugArgs {
    time: bool,
    display: Option<Pass>,
}

impl DebugArgs {
    pub fn set(time: bool, display: Option<Pass>) -> miette::Result<()> {
        DEBUG_ARGS
            .set(DebugArgs { time, display })
            .map_err(|_| miette!("Failed to set up `DebugArgs`."))
    }
}

pub static DEBUG_ARGS: OnceCell<DebugArgs> = OnceCell::new();

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Pass {
    Parse,
    Validate,
    Reveal,
    Atomize,
    Explicate,
    Select,
}
