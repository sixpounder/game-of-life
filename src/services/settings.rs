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

    pub fn window_width(&self) -> i32 {
        self.inner.int("window-width")
    }

    pub fn set_window_width(&self, value: i32) {
        self.inner.set_int("window-width", value).expect("Could not store window width");
    }

    pub fn window_height(&self) -> i32 {
        self.inner.int("window-height")
    }

    pub fn set_window_height(&self, value: i32) {
        self.inner.set_int("window-height", value).expect("Could not store window width");
    }

    pub fn fg_color(&self) -> String {
        self.inner.string("fg-color").to_string()
    }

    pub fn bg_color(&self) -> String {
        self.inner.string("bg-color").to_string()
    }

    pub fn fg_color_dark(&self) -> String {
        self.inner.string("fg-color-dark").to_string()
    }

    pub fn bg_color_dark(&self) -> String {
        self.inner.string("bg-color-dark").to_string()
    }
}
