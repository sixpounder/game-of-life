use gtk::{gio, glib};
use adw::{
    PreferencesWindow,
    subclass::{
        preferences_window::PreferencesWindowImpl,
        window::AdwWindowImpl
    }
};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
use crate::services::GameOfLifeSettings;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/preferences_window.ui")]
    pub struct GameOfLifePreferencesWindow {
        #[template_child]
        pub(super) evolution_speed: TemplateChild<gtk::SpinButton>,

        #[template_child]
        pub(super) draw_cells_outline: TemplateChild<gtk::Switch>,

        #[template_child]
        pub(super) evolution_speed_adjustment: TemplateChild<gtk::Adjustment>,

        #[template_child]
        pub(super) show_design_hint: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifePreferencesWindow {
        const NAME: &'static str = "GameOfLifePreferencesWindow";
        type Type = super::GameOfLifePreferencesWindow;
        type ParentType = PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifePreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
            obj.connect_events();
        }
    }

    impl WidgetImpl for GameOfLifePreferencesWindow {}
    impl WindowImpl for GameOfLifePreferencesWindow {}
    impl AdwWindowImpl for GameOfLifePreferencesWindow {}
    impl PreferencesWindowImpl for GameOfLifePreferencesWindow {}
}

glib::wrapper! {
    pub struct GameOfLifePreferencesWindow(ObjectSubclass<imp::GameOfLifePreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::PreferencesWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifePreferencesWindow {
    pub fn new(parent: Option<&gtk::Window>) -> Self {
        let this: Self = glib::Object::new(&[])
            .expect("Failed to create GameOfLifeNewUniverseView");

        this.set_transient_for(parent);

        this
    }

    fn setup_widgets(&self) {
        let settings = GameOfLifeSettings::default();
        self.imp().show_design_hint.set_active(settings.show_design_hint());
        self.imp().evolution_speed.set_value(f64::from(settings.evolution_speed()));
        self.imp().draw_cells_outline.set_active(settings.draw_cells_outline());
    }

    fn connect_events(&self) {
        self.imp().evolution_speed_adjustment.connect_notify_local(
            Some("value"),
            |adjustment, _| {
                let settings = GameOfLifeSettings::default();
                settings.set_evolution_speed(adjustment.value() as u32);
            }
        );

        self.imp().draw_cells_outline.connect_notify_local(
            Some("active"),
            |check, _| {
                let settings = GameOfLifeSettings::default();
                settings.set_draw_cells_outline(check.is_active());
            }
        );

        self.imp().show_design_hint.connect_notify_local(
            Some("active"),
            |check, _| {
                let settings = GameOfLifeSettings::default();
                settings.set_show_design_hint(check.is_active());
            }
        );
    }
}

