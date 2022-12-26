use crate::models::UniverseGridMode;
use gtk::{gio, glib};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_controls.ui")]
    pub struct GameOfLifeUniverseControls {
        #[template_child]
        pub(super) run_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub(super) save_snapshot_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub(super) random_seed_button: TemplateChild<gtk::Button>,

        pub(super) playing: std::cell::Cell<bool>,
        pub(super) editing: std::cell::Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeUniverseControls {
        const NAME: &'static str = "GameOfLifeUniverseControls";
        type Type = super::GameOfLifeUniverseControls;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifeUniverseControls {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("stopped", "", "", true, ParamFlags::READABLE),
                    ParamSpecBoolean::new("playing", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("editing", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("unfrozen", "", "", true, ParamFlags::READABLE),
                    ParamSpecString::new(
                        "run-button-icon-name",
                        "",
                        "",
                        Some("media-playback-start-symbolic"),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            let imp = obj.imp();
            match pspec.name() {
                "playing" => imp.playing.get().to_value(),
                "stopped" => (!imp.playing.get()).to_value(),
                "editing" => imp.editing.get().to_value(),
                "unfrozen" => (!imp.editing.get() && !imp.playing.get()).to_value(),
                "run-button-icon-name" => match obj.property("playing") {
                    true => "media-playback-stop-symbolic",
                    false => "media-playback-start-symbolic",
                }
                .to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "playing" => {
                    let now_playing = value.get::<bool>().unwrap();
                    let was_playing = self.playing.get();

                    if now_playing != was_playing {
                        self.playing.set(now_playing);

                        if now_playing {
                            obj.set_mode(UniverseGridMode::Run);
                        }

                        obj.notify("run-button-icon-name");
                        obj.notify("stopped");
                        obj.notify("unfrozen");
                    }
                }
                "editing" => {
                    obj.imp().editing.set(value.get::<bool>().unwrap());
                    obj.notify("unfrozen");
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for GameOfLifeUniverseControls {}
}

glib::wrapper! {
    pub struct GameOfLifeUniverseControls(ObjectSubclass<imp::GameOfLifeUniverseControls>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeUniverseControls {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new::<Self>(&[("application", application)])
    }

    pub fn set_mode(&self, mode: UniverseGridMode) {
        self.imp().editing.set(mode == UniverseGridMode::Design);
        self.notify("editing");
    }
}
