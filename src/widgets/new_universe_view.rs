use gtk::{gio, glib, glib::clone};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
use crate::services::GameOfLifeSettings;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NewUniverseType {
    Empty,
    Random,
    Template(&'static str)
}

impl Default for NewUniverseType {
    fn default() -> Self {
        NewUniverseType::Empty
    }
}

mod imp {
    use super::*;
    // use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString};
    // use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/new_universe_view.ui")]
    pub struct GameOfLifeNewUniverseView {
        #[template_child]
        pub(super) rows_entry: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub(super) columns_entry: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub(super) empty_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub(super) random_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub(super) template_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub(super) template_list_dropdown: TemplateChild<gtk::DropDown>,

        pub(super) option: std::cell::Cell<NewUniverseType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeNewUniverseView {
        const NAME: &'static str = "GameOfLifeNewUniverseView";
        type Type = super::GameOfLifeNewUniverseView;
        type ParentType = gtk::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifeNewUniverseView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
            obj.connect_events();
        }
    }

    impl WidgetImpl for GameOfLifeNewUniverseView {}
    impl WindowImpl for GameOfLifeNewUniverseView {}
    impl DialogImpl for GameOfLifeNewUniverseView {}
}

glib::wrapper! {
    pub struct GameOfLifeNewUniverseView(ObjectSubclass<imp::GameOfLifeNewUniverseView>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeNewUniverseView {
    pub fn new(parent: Option<&gtk::Window>) -> Self {
        let this: Self = glib::Object::new(&[])
            .expect("Failed to create GameOfLifeNewUniverseView");

        this.set_transient_for(parent);
        this.set_modal(true);

        this
    }

    fn setup_widgets(&self) {
        let settings = GameOfLifeSettings::default();

        let row_adjust = gtk::Adjustment::builder()
            .lower(10.)
            .upper(1000.)
            .step_increment(1.0)
            .page_increment(10.)
            .value(settings.universe_width().into())
            .build();

        let column_adjust = gtk::Adjustment::builder()
            .lower(10.)
            .upper(1000.)
            .step_increment(1.0)
            .page_increment(10.)
            .value(settings.universe_height().into())
            .build();

        row_adjust.connect_notify_local(
            Some("value"),
            clone!(@strong settings => move |adj, _| {
                settings.set_universe_width(adj.value() as i32);
            })
        );

        column_adjust.connect_notify_local(
            Some("value"),
            clone!(@strong settings => move |adj, _| {
                settings.set_universe_height(adj.value() as i32);
            })
        );
        self.imp().rows_entry.set_adjustment(&row_adjust);
        self.imp().columns_entry.set_adjustment(&column_adjust);

        self.imp().template_list_dropdown.set_sensitive(
            self.imp().template_check.is_active()
        );
    }

    fn connect_events(&self) {
        self.imp().empty_check.connect_toggled(
            clone!(@strong self as this => move |widget| {
                if widget.is_active() {
                    this.set_option(NewUniverseType::Empty);
                }
            })
        );

        self.imp().random_check.connect_toggled(
            clone!(@strong self as this => move |widget| {
                if widget.is_active() {
                    this.set_option(NewUniverseType::Random);
                }
            })
        );

        self.imp().template_check.connect_toggled(
            clone!(@strong self as this => move |widget| {
                if widget.is_active() {
                    let selected_template_object = this.imp().template_list_dropdown
                        .selected_item()
                        .expect("How?")
                        .downcast::<gtk::StringObject>()
                        .expect("How?")
                        .to_string();

                    match selected_template_object.as_str() {
                        "Glider" => { this.set_option(NewUniverseType::Template("glider")) },
                        "Pulsar" => { this.set_option(NewUniverseType::Template("pulsar")) },
                        "Circle of fire" => { this.set_option(NewUniverseType::Template("circle_of_fire")) },
                        "Quadpole" => { this.set_option(NewUniverseType::Template("quadpole")) },
                        "Spaceship" => { this.set_option(NewUniverseType::Template("spaceship")) },
                        _ => ()
                    }

                    this.imp().template_list_dropdown.set_sensitive(true);
                } else {
                    this.imp().template_list_dropdown.set_sensitive(false);
                }
            })
        );
    }

    pub fn option(&self) -> NewUniverseType {
        self.imp().option.get()
    }

    fn set_option(&self, value: NewUniverseType) {
        self.imp().option.set(value);
    }

    pub fn size(&self) -> (f64, f64) {
        (self.imp().rows_entry.value(), self.imp().columns_entry.value())
    }
}

