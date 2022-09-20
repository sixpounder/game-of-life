use adw::prelude::AdwApplicationExt;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, glib::clone, CompositeTemplate};

use crate::{
    models::UniverseGridMode,
    services::GameOfLifeSettings,
    config::APPLICATION_G_PATH
};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/window.ui")]
    pub struct GameOfLifeWindow {
        // Template widgets
        #[template_child]
        pub(super) header_bar: TemplateChild<gtk::HeaderBar>,

        #[template_child]
        pub(super) universe_grid: TemplateChild<crate::widgets::GameOfLifeUniverseGrid>,

        #[template_child]
        pub(super) controls: TemplateChild<crate::widgets::GameOfLifeUniverseControls>,

        pub(super) mode: std::cell::Cell<UniverseGridMode>,

        pub(super) provider: gtk::CssProvider,
        pub(super) settings: GameOfLifeSettings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeWindow {
        const NAME: &'static str = "GameOfLifeWindow";
        type Type = super::GameOfLifeWindow;
        type ParentType = gtk::ApplicationWindow;

        fn new() -> Self {
            Self {
                header_bar: TemplateChild::default(),
                universe_grid: TemplateChild::default(),
                controls: TemplateChild::default(),
                mode: std::cell::Cell::default(),
                provider: gtk::CssProvider::new(),
                settings: GameOfLifeSettings::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.play", None, move |win, _, _| {
                win.toggle_run();
            });

            klass.install_action("win.random-seed", None, move |win, _, _| {
                win.seed_universe();
            });

            klass.install_action("win.snapshot", None, move |win, _, _| {
                win.make_and_save_snapshot();
            });

            klass.install_action("win.toggle-design-mode", None, move |win, _, _| {
                win.toggle_edit_mode();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifeWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.connect_events();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        "run-button-icon-name",
                        "",
                        "",
                        Some("media-playback-start-symbolic"),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new("is-running", "", "", false, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-stopped", "", "", true, ParamFlags::READABLE),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "run-button-icon-name" => match obj.is_running() {
                    true => "media-playback-stop-symbolic",
                    false => "media-playback-start-symbolic",
                }
                .to_value(),
                "is-running" => obj.is_running().to_value(),
                "is-stopped" => (!obj.is_running()).to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GameOfLifeWindow {}
    impl WindowImpl for GameOfLifeWindow {}
    impl ApplicationWindowImpl for GameOfLifeWindow {}
}

glib::wrapper! {
    pub struct GameOfLifeWindow(ObjectSubclass<imp::GameOfLifeWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeWindow {
    pub fn new<P: glib::IsA<adw::Application>>(application: &P) -> Self {
        let win: Self = glib::Object::new(&[("application", application)])
            .expect("Failed to create GameOfLifeWindow");

        let style_manager = application.style_manager();

        win.setup_provider();
        win.update_prefers_dark_mode(style_manager.is_dark());

        style_manager.connect_dark_notify(glib::clone!(@strong win as this => move |sm| {
            this.update_prefers_dark_mode(sm.is_dark());
        }));

        win
    }

    pub fn mode(&self) -> UniverseGridMode {
        self.imp().mode.get()
    }

    pub fn set_mode(&self, value: UniverseGridMode) {
        self.imp().mode.set(value);
    }

    fn setup_provider(&self) {
        let imp = self.imp();
        imp.provider
            .load_from_resource(format!("{}/{}", APPLICATION_G_PATH, "style.css").as_str());
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(&display, &imp.provider, 400);
        }
    }

    fn connect_events(&self) {
        let imp = self.imp();

        // Updates buttons and other stuff when UniverseGrid running state changes
        imp.universe_grid.connect_notify_local(
            Some("is-running"),
            clone!(@strong self as this => move |_widget, _param| {
                this.notify("run-button-icon-name");
                this.notify("is-running");
                this.notify("is-stopped");
            }),
        );
    }

    pub fn is_running(&self) -> bool {
        let grid = &self.imp().universe_grid;

        if grid.is_bound() {
            self.imp().universe_grid.get().is_running()
        } else {
            false
        }
    }

    pub fn toggle_run(&self) {
        self.imp().universe_grid.toggle_run();
    }

    pub fn toggle_edit_mode(&self) {
        let grid = self.imp().universe_grid.get();
        let next_mode = match grid.mode() {
            UniverseGridMode::Design => UniverseGridMode::Run,
            UniverseGridMode::Run => UniverseGridMode::Design,
        };

        grid.set_mode(next_mode);

        let controls = self.imp().controls.get();
        controls.set_mode(next_mode);
    }

    fn make_and_save_snapshot(&self) {
        let snapshot = self.imp().universe_grid.get_universe_snapshot();
        todo!()
    }

    fn seed_universe(&self) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.random_seed();
    }

    fn update_prefers_dark_mode(&self, value: bool) {
        self.imp().universe_grid.get().set_prefers_dark_mode(value);
    }
}


