use crate::services::GameOfLifeSettings;
use adw::{
    subclass::{preferences_window::PreferencesWindowImpl, window::AdwWindowImpl},
    PreferencesWindow,
};
use gtk::{gdk::RGBA, gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/preferences_window.ui")]
    pub struct GameOfLifePreferencesWindow {
        #[template_child]
        pub(super) evolution_speed: TemplateChild<gtk::SpinButton>,

        #[template_child]
        pub(super) cell_color_picker: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub(super) background_color_picker: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub(super) cell_color_dark_picker: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub(super) background_color_dark_picker: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub(super) draw_cells_outline: TemplateChild<gtk::Switch>,

        #[template_child]
        pub(super) fade_out_dead_cells: TemplateChild<gtk::Switch>,

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
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_bindings();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        "universe-cell-color",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "universe-background-color",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "universe-cell-color-dark",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "universe-background-color-dark",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "universe-cell-color" => {
                    let str_value = value.get::<String>().unwrap();
                    let rgba_value = RGBA::parse(str_value.as_str()).unwrap();
                    self.cell_color_picker.set_rgba(&rgba_value);
                    // self.settings.set_fg_color(rgba_value.to_string());
                }
                "universe-background-color" => {
                    let str_value = value.get::<String>().unwrap();
                    let rgba_value = RGBA::parse(str_value.as_str()).unwrap();
                    self.background_color_picker.set_rgba(&rgba_value);
                    // self.settings.set_fg_color(rgba_value.to_string());
                }
                "universe-cell-color-dark" => {
                    let str_value = value.get::<String>().unwrap();
                    let rgba_value = RGBA::parse(str_value.as_str()).unwrap();
                    self.cell_color_dark_picker.set_rgba(&rgba_value);
                    // self.settings.set_fg_color(rgba_value.to_string());
                }
                "universe-background-color-dark" => {
                    let str_value = value.get::<String>().unwrap();
                    let rgba_value = RGBA::parse(str_value.as_str()).unwrap();
                    self.background_color_dark_picker.set_rgba(&rgba_value);
                    // self.settings.set_fg_color(rgba_value.to_string());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "universe-cell-color" => self.cell_color_picker.rgba().to_string().to_value(),
                "universe-background-color" => {
                    self.background_color_picker.rgba().to_string().to_value()
                }
                "universe-cell-color-dark" => {
                    self.cell_color_dark_picker.rgba().to_string().to_value()
                }
                "universe-background-color-dark" => self
                    .background_color_dark_picker
                    .rgba()
                    .to_string()
                    .to_value(),
                _ => unimplemented!(),
            }
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
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[])
    }

    fn setup_bindings(&self) {
        let settings = GameOfLifeSettings::default();
        let imp = self.imp();
        let instance = imp.obj();

        settings.bind(
            "draw-cells-outline",
            &imp.draw_cells_outline.get(),
            "active",
        );
        settings.bind(
            "fade-out-cells",
            &imp.fade_out_dead_cells.get(),
            "active",
        );
        settings.bind(
            "allow-render-during-resize",
            &imp.allow_render_on_resize.get(),
            "active",
        );
        settings.bind("show-design-hint", &imp.show_design_hint.get(), "active");
        settings.bind(
            "evolution-speed",
            &imp.evolution_speed_adjustment.get(),
            "value",
        );

        // Proxy colors to this widget, to convert from RGBA to string
        settings.bind("fg-color", instance.as_ref(), "universe-cell-color");
        settings.bind("bg-color", instance.as_ref(), "universe-background-color");
        settings.bind(
            "fg-color-dark",
            instance.as_ref(),
            "universe-cell-color-dark",
        );
        settings.bind(
            "bg-color-dark",
            instance.as_ref(),
            "universe-background-color-dark",
        );

        // Listen for color pickers

        imp.cell_color_picker.connect_color_set(
            glib::clone!(@strong self as this => move |picker| {
                this.set_property("universe-cell-color", picker.rgba().to_string().to_value());
            }),
        );

        imp.background_color_picker.connect_color_set(
            glib::clone!(@strong self as this => move |picker| {
                this.set_property("universe-background-color", picker.rgba().to_string().to_value());
            })
        );

        imp.cell_color_dark_picker.connect_color_set(
            glib::clone!(@strong self as this => move |picker| {
                this.set_property("universe-cell-color-dark", picker.rgba().to_string().to_value());
            }),
        );

        imp.background_color_dark_picker.connect_color_set(
            glib::clone!(@strong self as this => move |picker| {
                this.set_property("universe-background-color-dark", picker.rgba().to_string().to_value());
            })
        );
    }
}
