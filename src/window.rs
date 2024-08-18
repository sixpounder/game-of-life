use std::{io::prelude::*, str::FromStr};

use crate::i18n::i18n;
use adw::prelude::AdwApplicationExt;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, glib::clone, CompositeTemplate};
use glib::prelude::IsA;

use crate::{
    config::{APPLICATION_G_PATH, G_LOG_DOMAIN},
    models::{Universe, UniverseGridMode, UniverseSnapshot},
    services::{GameOfLifeSettings, Template},
    widgets::{GameOfLifeNewUniverseView, NewUniverseType},
};

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/window.ui")]
    pub struct GameOfLifeWindow {
        // Template widgets
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub(super) universe_grid: TemplateChild<crate::widgets::GameOfLifeUniverseGrid>,

        #[template_child]
        pub(super) controls: TemplateChild<crate::widgets::GameOfLifeUniverseControls>,

        pub(super) mode: std::cell::Cell<UniverseGridMode>,

        pub(super) provider: gtk::CssProvider,

        pub(super) style_manager: adw::StyleManager,

        pub(super) settings: GameOfLifeSettings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeWindow {
        const NAME: &'static str = "GameOfLifeWindow";
        type Type = super::GameOfLifeWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                toast_overlay: TemplateChild::default(),
                universe_grid: TemplateChild::default(),
                controls: TemplateChild::default(),
                mode: std::cell::Cell::default(),
                provider: gtk::CssProvider::new(),
                settings: GameOfLifeSettings::default(),
                style_manager: adw::StyleManager::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.new", None, move |win, _, _| {
                win.new_universe_dialog();
            });

            klass.install_action("win.random-seed", None, move |win, _, _| {
                win.seed_universe();
            });

            klass.install_action("win.skip-forward-one", None, move |win, _, _| {
                win.skip_forward_one();
            });

            klass.install_action("win.rewind-one", None, move |win, _, _| {
                win.rewind_one();
            });

            klass.install_action("win.play", None, move |win, _, _| {
                win.toggle_run();
            });

            klass.install_action("win.snapshot", None, move |win, _, _| {
                win.make_and_save_snapshot();
            });

            klass.install_action("win.open-snapshot", None, move |win, _, _| {
                win.select_and_load_snapshot();
            });

            klass.install_action("win.toggle-design-mode", None, move |win, _, _| {
                win.toggle_edit_mode();
            });

            klass.install_action("win.toggle-instrument-brush", None, move |win, _, _| {
                win.toggle_instrument_brush();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifeWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_provider();
            obj.setup_widgets();
            obj.restore_window_state();
            obj.connect_events();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("run-button-icon-name")
                        .default_value(Some("media-playback-start-symbolic"))
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("running")
                        .default_value(false)
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("stopped")
                        .default_value(true)
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("allow-render-on-resize")
                        .default_value(false)
                        .read_only()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "run-button-icon-name" => match obj.is_running() {
                    true => "media-playback-stop-symbolic",
                    false => "media-playback-start-symbolic",
                }
                .to_value(),
                "running" => obj.is_running().to_value(),
                "stopped" => (!obj.is_running()).to_value(),
                "allow-render-on-resize" => self.settings.allow_render_during_resize().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GameOfLifeWindow {}
    impl WindowImpl for GameOfLifeWindow {}
    impl ApplicationWindowImpl for GameOfLifeWindow {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for GameOfLifeWindow {}
}

glib::wrapper! {
    pub struct GameOfLifeWindow(ObjectSubclass<imp::GameOfLifeWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeWindow {
    pub fn new<P: IsA<adw::Application>>(application: &P) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();

        let style_manager = application.style_manager();

        win.update_widgets();

        style_manager.connect_dark_notify(glib::clone!(@strong win as this => move |_sm| {
            this.update_widgets();
        }));

        win
    }

    pub fn mode(&self) -> UniverseGridMode {
        self.imp().mode.get()
    }

    pub fn set_mode(&self, value: UniverseGridMode) {
        self.imp().mode.set(value);
    }

    fn setup_widgets(&self) {
        let settings = &self.imp().settings;
        let grid = self.imp().universe_grid.get();
        grid.set_allow_render_on_resize(settings.allow_render_during_resize());
        grid.set_evolution_speed(settings.evolution_speed());
        grid.set_draw_cells_outline(settings.draw_cells_outline());
        grid.set_animated(settings.animated());
    }

    fn setup_provider(&self) {
        let imp = self.imp();
        imp.provider
            .load_from_resource(format!("{}/{}", APPLICATION_G_PATH, "style.css").as_str());
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(&display, &imp.provider, 400);
        }
    }

    fn connect_events(&self) {
        let imp = self.imp();
        let settings = &imp.settings;

        // Updates buttons and other stuff when UniverseGrid running state changes
        imp.universe_grid.connect_notify_local(
            Some("running"),
            clone!(@strong self as this => move |_widget, _param| {
                this.notify("run-button-icon-name");
                this.notify("running");
                this.notify("stopped");
            }),
        );

        settings.connect_changed(
            "draw-cells-outline",
            clone!(@strong self as this, @strong settings as s => move |_,_| {
                this.imp().universe_grid.set_draw_cells_outline(s.draw_cells_outline())
            }),
        );

        settings.connect_changed(
            "fade-out-cells",
            clone!(@strong self as this, @strong settings as s => move |_,_| {
                this.imp().universe_grid.set_animated(s.animated())
            }),
        );

        settings.connect_changed(
            "evolution-speed",
            clone!(@strong self as this, @strong settings as s => move |_,_| {
                this.imp().universe_grid.set_evolution_speed(s.evolution_speed())
            }),
        );

        settings.connect_changed("allow-render-during-resize",
            clone!(@strong self as this, @strong settings as s => move |_,_| {
                this.imp().universe_grid.set_allow_render_on_resize(s.allow_render_during_resize());
            })
        );

        settings.connect_changed(
            "fg-color",
            clone!(@strong self as this, @strong settings as s => move |_, _| {
                this.update_widgets();
            }),
        );

        settings.connect_changed(
            "bg-color",
            clone!(@strong self as this, @strong settings as s => move |_, _| {
                this.update_widgets();
            }),
        );

        settings.connect_changed(
            "fg-color-dark",
            clone!(@strong self as this, @strong settings as s => move |_, _| {
                this.update_widgets();
            }),
        );

        settings.connect_changed(
            "bg-color-dark",
            clone!(@strong self as this, @strong settings as s => move |_, _| {
                this.update_widgets();
            }),
        );

        self.connect_close_request(move |window| {
            glib::g_debug!(G_LOG_DOMAIN, "Saving window state");
            let width = window.default_size().0;
            let height = window.default_size().1;
            let settings = GameOfLifeSettings::default();
            settings.set_window_width(width);
            settings.set_window_height(height);
            glib::Propagation::Proceed
        });
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
        self.notify("running");
    }

    pub fn toggle_edit_mode(&self) {
        let controls = self.imp().controls.get();
        let tools_revealed = controls.tools_revealed();
        controls.set_tools_revealed(!tools_revealed);
    }

    pub fn toggle_instrument_brush(&self) {
        let settings = &self.imp().settings;
        let controls = self.imp().controls.get();

        controls.toggle_brush();

        if controls.brush() {
            self.imp()
                .universe_grid
                .set_mode(UniverseGridMode::Unlocked);
            if settings.show_design_hint() {
                let msg = i18n("Left click to make a cell alive, right click to make it dead");
                let toast = adw::Toast::new(&msg);
                toast.set_action_name(Some("app.disable-design-hint"));
                toast.set_button_label(Some(i18n("Do not show again").as_str()));
                self.imp().toast_overlay.add_toast(toast);
            }
        } else {
            self.imp().universe_grid.set_mode(UniverseGridMode::Locked);
        }
    }

    fn make_and_save_snapshot(&self) {
        let app = gio::Application::default()
            .expect("Failed to retrieve application singleton")
            .downcast::<gtk::Application>()
            .unwrap();
        let win = app
            .active_window()
            .unwrap()
            .downcast::<gtk::Window>()
            .unwrap();

        let dialog = gtk::FileChooserNative::builder()
            .accept_label(&i18n("_Save"))
            .cancel_label(&i18n("_Cancel"))
            .modal(true)
            .title(&i18n("Save universe snapshot"))
            .transient_for(&win)
            .select_multiple(false)
            .action(gtk::FileChooserAction::Save)
            .build();

        dialog.connect_response(
            clone!(@strong dialog, @weak self as win => move |_, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(file) = dialog.file().as_ref() {
                        let snapshot = win.imp().universe_grid.get_universe_snapshot();
                        match snapshot.serialize() {
                            Ok(serialized) => {
                                let file_io_stream = if file.query_exists(gtk::gio::Cancellable::NONE) {
                                    file.open_readwrite(gtk::gio::Cancellable::NONE).unwrap()
                                } else {
                                    file.create_readwrite(gtk::gio::FileCreateFlags::NONE | gtk::gio::FileCreateFlags::REPLACE_DESTINATION, gtk::gio::Cancellable::NONE).unwrap()
                                };

                                let write_result = file_io_stream.output_stream().write_all(serialized.as_slice(), gtk::gio::Cancellable::NONE);
                                match write_result {
                                    Ok((bytes_written, _)) => {
                                        glib::info!("Written {} bytes", bytes_written);
                                    },
                                    Err(error) => {
                                        win.add_toast(i18n("Unable to write to file"));
                                         glib::g_critical!(G_LOG_DOMAIN, "Unable to write to file: {}", error);
                                    }
                                }
                            },
                            Err(error) => {
                                win.add_toast(i18n("Unable to serialize snapshot"));
                                 glib::g_critical!(G_LOG_DOMAIN, "Unable to serialize universe snapshot: {}", error);
                            }
                        }
                    }
                }
            })
        );

        dialog.show();
    }

    fn select_and_load_snapshot(&self) {
        let app = gio::Application::default()
            .expect("Failed to retrieve application singleton")
            .downcast::<gtk::Application>()
            .unwrap();
        let win = app
            .active_window()
            .unwrap()
            .downcast::<gtk::Window>()
            .unwrap();

        let dialog = gtk::FileChooserNative::builder()
            .accept_label(&i18n("_Open"))
            .cancel_label(&i18n("_Cancel"))
            .modal(true)
            .title(&i18n("Open universe snapshot"))
            .transient_for(&win)
            .select_multiple(false)
            .action(gtk::FileChooserAction::Open)
            .build();

        dialog.connect_response(
            clone!(@strong dialog, @weak self as win => move |_, response| {
                let file = dialog.file();
                if response == gtk::ResponseType::Accept {
                    if let Some(file) = file.as_ref() {
                        if file.query_exists(gio::Cancellable::NONE) {
                            let mut buffer: Vec<u8> = vec![];

                            let file_io_stream = dialog.file().unwrap();
                            let file_name = file_io_stream.path().unwrap();
                            let file_name = file_name.to_str().unwrap();

                            if let Ok(file) = std::fs::File::open(file_name) {
                                let mut file = std::io::BufReader::new(file);
                                if let Ok(bytes_read) = file.read_to_end(&mut buffer) {
                                    glib::debug!("Opening snapshot (read {} bytes)", bytes_read);

                                    match UniverseSnapshot::try_from(&buffer) {
                                        Ok(snapshot) => {
                                            win.seed_from_snapshot(snapshot);
                                        },
                                        Err(error) => {
                                            glib::g_critical!(G_LOG_DOMAIN, "Unreadable file: {:?}", error);
                                            win.add_toast(i18n("Unreadable file"));
                                        }
                                    }
                                } else {
                                    // Failed to read file
                                    glib::g_critical!(G_LOG_DOMAIN, "Unreadable file",);
                                    win.add_toast(i18n("Unreadable file"));
                                }
                            } else {
                                // File not accessible
                                glib::g_critical!(G_LOG_DOMAIN, "File not accessible",);
                                win.add_toast(i18n("File not existing or not accessible"));
                            }
                        }
                    }
                }
            })
        );

        dialog.show();
    }

    fn new_universe_dialog(&self) {
        let app = gio::Application::default()
            .expect("Failed to retrieve application singleton")
            .downcast::<gtk::Application>()
            .unwrap();
        let win = app
            .active_window()
            .unwrap()
            .downcast::<gtk::Window>()
            .unwrap();
        let dialog = GameOfLifeNewUniverseView::new();
        dialog.set_modal(true);
        dialog.set_transient_for(Some(&win));

        dialog.connect_response(
            clone!(@strong dialog, @weak self as win => move |_, response| {
                if response == gtk::ResponseType::Ok {
                    let (target_w, target_h) = dialog.size();
                    match dialog.option() {
                        NewUniverseType::Empty => win.new_empty(target_w as usize, target_h as usize),
                        NewUniverseType::Random => win.new_random(target_w as usize, target_h as usize),
                        NewUniverseType::Template(template_name) => {
                            glib::debug!("Seeding from {} template", template_name);
                            match Template::read_template(template_name) {
                                Ok(read) => {
                                    match UniverseSnapshot::try_from(&read) {
                                        Ok(snapshot) => {
                                            win.seed_from_snapshot(snapshot);
                                        },
                                        Err(error) => {
                                            glib::g_critical!(G_LOG_DOMAIN, "Unreadable template: {:?}", error);
                                            win.add_toast(i18n("Bad template data"));
                                        }
                                    }
                                },
                                Err(error) => {
                                    glib::g_critical!(G_LOG_DOMAIN, "Could not load template: {}", error);
                                    win.add_toast(i18n("Template not found"));
                                }
                            }
                        }
                    }
                }
                dialog.close();
            })
        );
        dialog.show();
    }

    fn new_empty(&self, rows: usize, columns: usize) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.set_universe(Universe::new_empty(rows, columns));
    }

    fn new_random(&self, rows: usize, columns: usize) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.set_universe(Universe::new_random(rows, columns));
    }

    fn seed_universe(&self) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.random_seed();
    }

    fn skip_forward_one(&self) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.skip_forward_one();
    }

    fn rewind_one(&self) {
        let universe_grid = self.imp().universe_grid.get();
        universe_grid.rewind_one();
    }

    fn seed_from_snapshot(&self, snapshot: UniverseSnapshot) {
        let universe_grid = self.imp().universe_grid.get();
        let universe = snapshot.into();
        universe_grid.set_universe(universe);
    }

    fn update_widgets(&self) {
        let style_manager = &self.imp().style_manager;
        let settings = &self.imp().settings;
        let grid = self.imp().universe_grid.get();
        let (cell_color, background_color);

        if style_manager.is_dark() {
            cell_color = settings.fg_color_dark();
            background_color = settings.bg_color_dark();
        } else {
            cell_color = settings.fg_color();
            background_color = settings.bg_color();
        }

        grid.set_cell_color(Some(gtk::gdk::RGBA::from_str(&cell_color).unwrap()));
        grid.set_background_color(Some(gtk::gdk::RGBA::from_str(&background_color).unwrap()));
    }

    fn restore_window_state(&self) {
        let settings = &self.imp().settings;
        self.set_default_size(settings.window_width(), settings.window_height());
    }

    fn add_toast(&self, msg: String) {
        let toast = adw::Toast::new(&msg);
        self.imp().toast_overlay.add_toast(toast);
    }
}
