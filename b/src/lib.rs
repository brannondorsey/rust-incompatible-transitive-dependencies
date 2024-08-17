// Required for the outdated 0.3.* version of log
#[macro_use]
extern crate log;

use log::info;

pub fn log() {
    info!("logged using log@0.3.9");
}
