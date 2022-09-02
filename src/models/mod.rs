mod universe;

pub use universe::*;

use gtk::glib;

#[derive(Clone, Debug, glib::Enum, Copy)]
#[enum_type(name = "UniverseGridMode")]
pub enum UniverseGridMode {
    Design,
    Run
}

impl Default for UniverseGridMode {
    fn default() -> Self {
        Self::Design
    }
}

