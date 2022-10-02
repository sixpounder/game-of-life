use gtk::{gio, glib};
use adw::{
    PreferencesWindow,
    subclass::{
        preferences_window::PreferencesWindowImpl,
        window::AdwWindowImpl
    }
};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate, glib::IsA};
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
        pub(super) allow_render_on_resize: TemplateChild<gtk::Switch>,

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
            obj.setup_bindings();
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

    fn setup_bindings(&self) {
        let settings = GameOfLifeSettings::default();
        let imp = self.imp();

        settings.bind("draw-cells-outline", imp.draw_cells_outline.get(), "active");
        settings.bind("allow-render-during-resize", imp.allow_render_on_resize.get(), "active");
        settings.bind("show-design-hint", imp.show_design_hint.get(), "active");
        settings.bind("evolution-speed", imp.evolution_speed_adjustment.get(), "value");
    }
}

