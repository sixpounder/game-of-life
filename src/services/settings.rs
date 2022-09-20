use gtk::gio::prelude::SettingsExt;
use crate::config::APPLICATION_ID;

#[derive(Debug)]
pub struct GameOfLifeSettings {
    inner: gtk::gio::Settings
}

impl Default for GameOfLifeSettings {
    fn default() -> Self {
        Self {
            inner: gtk::gio::Settings::new(APPLICATION_ID)
        }
    }
}

impl GameOfLifeSettings {
    pub fn set_evolution_speed(&self, value: u32) {
        self.inner.set_uint("evolution-speed", value).expect("Could not set evolution speed");
    }
    pub fn evolution_speed(&self) -> u32 {
        self.inner.uint("evolution-speed")
    }
}
