use crate::models::UniverseGridMode;
use crate::widgets::UniverseGridRequest;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, glib::clone, CompositeTemplate};

use crate::config::{APPLICATION_G_PATH, APPLICATION_ID};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/window.ui")]
    pub struct GameOfLifeWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,

        #[template_child]
        pub universe_grid: TemplateChild<crate::widgets::GameOfLifeUniverseGrid>,

        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub save_snapshot_button: TemplateChild<gtk::Button>,

        pub(crate) mode: std::cell::Cell<UniverseGridMode>,

        pub provider: gtk::CssProvider,
        // pub settings: gtk::gio::Settings,
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
                run_button: TemplateChild::default(),
                save_snapshot_button: TemplateChild::default(),
                mode: std::cell::Cell::default(),
                provider: gtk::CssProvider::new(),
                // settings: gtk::gio::Settings::with_path(
                //     APPLICATION_ID,
                //     format!("{}/", APPLICATION_G_PATH).as_str(),
                // ),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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
                    ParamSpecBoolean::new("can-snapshot", "", "", true, ParamFlags::READABLE)
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
                "can-snapshot" => (!obj.is_running()).to_value(),
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
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        let win: Self = glib::Object::new(&[("application", application)])
            .expect("Failed to create GameOfLifeWindow");

        win.setup_provider();

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
        imp
            .run_button
            .connect_clicked(clone!(@strong self as this => move |_widget| {
                this.toggle_run();
            }));

        imp.save_snapshot_button.connect_clicked(
            clone!(@strong self as this => move |_widget| {
                this.make_and_save_snapshot();
            })
        );

        // Updates buttons and other stuff when UniverseGrid running state changes
        imp.universe_grid.connect_notify_local(
            Some("is-running"),
            clone!(@strong self as this => move |_widget, _param| {
                this.notify("run-button-icon-name");
                this.notify("is-running");
                this.notify("can-snapshot");
            }),
        );
    }

    pub fn is_running(&self) -> bool {
        let grid = &self.imp().universe_grid;

        if grid.is_bound() {
            self.imp().universe_grid.is_running()
        } else {
            false
        }
    }

    pub fn toggle_run(&self) {
        let sender = self.imp().universe_grid.get_sender();
        match self.is_running() {
            false => sender.send(UniverseGridRequest::Run).unwrap(),
            true => sender.send(UniverseGridRequest::Halt).unwrap(),
        }
    }

    fn make_and_save_snapshot(&self) {
        let snapshot = self.imp().universe_grid.get_universe_snapshot();
    }
}


