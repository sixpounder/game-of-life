use crate::config::{APPLICATION_ID, G_LOG_DOMAIN};
use gtk::gdk;
use gtk::gio::prelude::{SettingsExt, SettingsExtManual};
use gtk::glib::IsA;

#[derive(Debug, Clone)]
pub struct GameOfLifeSettings {
    inner: gtk::gio::Settings,
}

impl Default for GameOfLifeSettings {
    fn default() -> Self {
        Self {
            inner: gtk::gio::Settings::new(APPLICATION_ID),
        }
    }
}

impl GameOfLifeSettings {
    #[allow(dead_code)]
    pub fn set_evolution_speed(&self, value: u32) {
        self.inner
            .set_uint("evolution-speed", value)
            .expect("Could not set evolution speed");
    }

    pub fn evolution_speed(&self) -> u32 {
        self.inner.uint("evolution-speed")
    }

    pub fn window_width(&self) -> i32 {
        self.inner.int("window-width")
    }

    pub fn set_window_width(&self, value: i32) {
        self.inner
            .set_int("window-width", value)
            .expect("Could not store window width");
    }

    pub fn window_height(&self) -> i32 {
        self.inner.int("window-height")
    }

    pub fn set_window_height(&self, value: i32) {
        self.inner
            .set_int("window-height", value)
            .expect("Could not store window width");
    }

    pub fn fg_color(&self) -> String {
        self.inner.string("fg-color").to_string()
    }

    #[allow(dead_code)]
    pub fn fg_color_rgba(&self) -> gdk::RGBA {
        gdk::RGBA::parse(self.fg_color().as_str()).expect("Cannot parse RGBA")
    }

    #[allow(dead_code)]
    pub fn set_fg_color(&self, value: String) {
        self.inner
            .set_string("fg-color", value.as_str())
            .expect("Could not store fg-color preference");
    }

    pub fn bg_color(&self) -> String {
        self.inner.string("bg-color").to_string()
    }

    #[allow(dead_code)]
    pub fn set_bg_color(&self, value: String) {
        self.inner
            .set_string("bg-color", value.as_str())
            .expect("Could not store bg-color preference");
    }

    pub fn fg_color_dark(&self) -> String {
        self.inner.string("fg-color-dark").to_string()
    }

    #[allow(dead_code)]
    pub fn set_fg_color_dark(&self, value: String) {
        self.inner
            .set_string("fg-color-dark", value.as_str())
            .expect("Could not store fg-color-dark preference");
    }

    pub fn bg_color_dark(&self) -> String {
        self.inner.string("bg-color-dark").to_string()
    }

    #[allow(dead_code)]
    pub fn set_bg_color_dark(&self, value: String) {
        self.inner
            .set_string("bg-color-dark", value.as_str())
            .expect("Could not store bg-color-dark preference");
    }

    pub fn universe_width(&self) -> i32 {
        self.inner.int("universe-width")
    }

    pub fn set_universe_width(&self, value: i32) {
        self.inner
            .set_int("universe-width", value)
            .expect("Could not store default universe width");
    }

    pub fn universe_height(&self) -> i32 {
        self.inner.int("universe-height")
    }

    pub fn set_universe_height(&self, value: i32) {
        self.inner
            .set_int("universe-height", value)
            .expect("Could not store default universe height");
    }

    pub fn draw_cells_outline(&self) -> bool {
        self.inner.boolean("draw-cells-outline")
    }

    pub fn fade_out_cells(&self) -> bool {
        self.inner.boolean("fade-out-cells")
    }

    #[allow(dead_code)]
    pub fn set_draw_cells_outline(&self, value: bool) {
        self.inner
            .set_boolean("draw-cells-outline", value)
            .expect("Could not store cells outline preference")
    }

    pub fn show_design_hint(&self) -> bool {
        self.inner.boolean("show-design-hint")
    }

    pub fn set_show_design_hint(&self, value: bool) {
        self.inner
            .set_boolean("show-design-hint", value)
            .expect("Could not store design hint preference")
    }

    pub fn allow_render_during_resize(&self) -> bool {
        self.inner.boolean("allow-render-during-resize")
    }

    #[allow(dead_code)]
    pub fn set_allow_render_during_resize(&self, value: bool) {
        self.inner
            .set_boolean("allow-render-during-resize", value)
            .expect("Coult not store allow render during resize preference")
    }

    pub fn connect_changed<F>(&self, key: &str, f: F)
    where
        F: Fn(&gtk::gio::Settings, &str) + 'static,
    {
        self.inner.connect_changed(Some(key), move |settings, key| {
            glib::info!("GSettings:{} changed", key);
            f(settings, key);
        });
    }

    pub fn bind<P>(&self, key: &str, object: &P, property: &str)
    where
        P: IsA<glib::Object>,
    {
        self.inner.bind(key, object, property).build();
    }
}
