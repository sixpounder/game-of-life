use gtk::gio::prelude::SettingsExt;
use crate::config::{APPLICATION_ID, G_LOG_DOMAIN};

#[derive(Debug, Clone)]
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

    pub fn universe_width(&self) -> i32 {
        self.inner.int("universe-width")
    }

    pub fn set_universe_width(&self, value: i32) {
        self.inner.set_int("universe-width", value).expect("Could not store default universe width");
    }

    pub fn universe_height(&self) -> i32 {
        self.inner.int("universe-height")
    }

    pub fn set_universe_height(&self, value: i32) {
        self.inner.set_int("universe-height", value).expect("Could not store default universe height");
    }

    pub fn draw_cells_outline(&self) -> bool {
        self.inner.boolean("draw-cells-outline")
    }

    pub fn set_draw_cells_outline(&self, value: bool) {
        self.inner.set_boolean("draw-cells-outline", value).expect("Could not store cells outline preference")
    }

    pub fn show_design_hint(&self) -> bool {
        self.inner.boolean("show-design-hint")
    }

    pub fn set_show_design_hint(&self, value: bool) {
        self.inner.set_boolean("show-design-hint", value).expect("Could not store design hint preference")
    }

    pub fn connect_changed<F>(&self, key: &str, f: F)
    where
        F: Fn(&gtk::gio::Settings, &str) + 'static
    {
        self.inner.connect_changed(Some(key), move |settings, key| {
            glib::info!("GSettings:{} changed", key);
            f(settings, key);
        });
    }
}
