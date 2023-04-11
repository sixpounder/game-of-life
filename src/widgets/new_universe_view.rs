use crate::{config::G_LOG_DOMAIN, services::GameOfLifeSettings};
use gtk::{gio, glib, glib::clone};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NewUniverseType {
    Empty,
    Random,
    Template(&'static str),
}

impl Default for NewUniverseType {
    fn default() -> Self {
        NewUniverseType::Empty
    }
}

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoolean};
    use once_cell::sync::Lazy;

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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_widgets();
            obj.connect_events();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecBoolean::builder("dimensions-editable")
                    .default_value(false)
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "dimensions-editable" => {
                    (!matches!(obj.option(), NewUniverseType::Template(_))).to_value()
                }
                _ => unimplemented!(),
            }
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

impl Default for GameOfLifeNewUniverseView {
    fn default() -> Self {
        Self::new()
    }
}

impl GameOfLifeNewUniverseView {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
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
            }),
        );

        column_adjust.connect_notify_local(
            Some("value"),
            clone!(@strong settings => move |adj, _| {
                settings.set_universe_height(adj.value() as i32);
            }),
        );
        self.imp().rows_entry.set_adjustment(&row_adjust);
        self.imp().columns_entry.set_adjustment(&column_adjust);

        self.imp()
            .template_list_dropdown
            .set_sensitive(self.imp().template_check.is_active());
    }

    fn connect_events(&self) {
        self.imp()
            .template_check
            .connect_toggled(clone!(@strong self as this => move |widget| {
                this.imp().template_list_dropdown.set_sensitive(widget.is_active());
                this.notify("dimensions-editable");
            }));
    }

    pub fn option(&self) -> NewUniverseType {
        if self.imp().empty_check.is_active() {
            NewUniverseType::Empty
        } else if self.imp().random_check.is_active() {
            NewUniverseType::Random
        } else {
            let selected_template_object = self
                .imp()
                .template_list_dropdown
                .selected_item()
                .expect("How?")
                .downcast::<gtk::StringObject>()
                .expect("How?")
                .string();
            glib::g_debug!(
                G_LOG_DOMAIN,
                "Selected {} template",
                selected_template_object.as_str()
            );
            match selected_template_object.as_str() {
                "Glider" => NewUniverseType::Template("glider"),
                "Pulsar" => NewUniverseType::Template("pulsar"),
                "Circle of fire" => NewUniverseType::Template("circle_of_fire"),
                "Quadpole" => NewUniverseType::Template("quadpole"),
                "Spaceship" => NewUniverseType::Template("spaceship"),
                _ => unreachable!("This should not happen"),
            }
        }
    }

    pub fn size(&self) -> (f64, f64) {
        (
            self.imp().rows_entry.value(),
            self.imp().columns_entry.value(),
        )
    }
}
