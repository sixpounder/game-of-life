use adw::subclass::prelude::*;
use adw::prelude::*;
use glib::clone;
use gtk::{gio, glib};

use crate::config::{APPLICATION_ID, VERSION};
use crate::i18n::translators_list;
use crate::{services::GameOfLifeSettings, widgets::GameOfLifePreferencesWindow, GameOfLifeWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct GameOfLifeApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeApplication {
        const NAME: &'static str = "GameOfLifeApplication";
        type Type = super::GameOfLifeApplication;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for GameOfLifeApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.preferences", &["<ctrl>comma"]);
            obj.set_accels_for_action("win.play", &["space"]);
            obj.set_accels_for_action("win.snapshot", &["<ctrl>s"]);
            obj.set_accels_for_action("win.open-snapshot", &["<ctrl>o"]);
            obj.set_accels_for_action("win.toggle-design-mode", &["e"]);
            obj.set_accels_for_action("win.new", &["<ctrl>n"]);
            obj.set_accels_for_action("win.new-empty", &["<ctrl>e"]);
            obj.set_accels_for_action("win.random-seed", &["<ctrl>r"]);
        }
    }

    impl ApplicationImpl for GameOfLifeApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            // Get the current window or create one if necessary
            let application = self.obj();
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = GameOfLifeWindow::new(application.as_ref());
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for GameOfLifeApplication {}
    impl AdwApplicationImpl for GameOfLifeApplication {}
}

glib::wrapper! {
    pub struct GameOfLifeApplication(ObjectSubclass<imp::GameOfLifeApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        // glib::Object::new::<Self>(&[("application-id", &application_id), ("flags", flags)])
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.quit();
            }
        ));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.show_about();
            }
        ));
        self.add_action(&about_action);

        let preferences_action = gio::SimpleAction::new("preferences", None);
        preferences_action.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                let preferences_window = GameOfLifePreferencesWindow::new();
                preferences_window.set_transient_for(app.active_window().as_ref());
                preferences_window.set_modal(false);
                preferences_window.show();
            }
        ));
        self.add_action(&preferences_action);

        let disable_design_hint_action = gio::SimpleAction::new("disable-design-hint", None);
        disable_design_hint_action.connect_activate(clone!(
            #[weak(rename_to = _app)]
            self,
            move |_, _| {
                GameOfLifeSettings::default().set_show_design_hint(false);
            }
        ));
        self.add_action(&disable_design_hint_action);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = adw::AboutDialog::builder()
            .application_name("Game of Life")
            .developer_name("Andrea Coronese")
            .developers(vec!["Andrea Coronese"])
            .copyright("© 2022 Andrea Coronese")
            .application_icon(APPLICATION_ID)
            .website("https://flathub.org/apps/details/com.github.sixpounder.GameOfLife")
            .issue_url("https://github.com/sixpounder/game-of-life/issues")
            .version(VERSION)
            .license_type(gtk::License::Gpl30)
            .translator_credits(translators_list().join("\n").as_str())
            .build();

        dialog.present(Some(&window));
    }
}

