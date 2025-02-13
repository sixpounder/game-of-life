use gtk::{gio, glib};
use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_controls.ui")]
    pub struct GameOfLifeUniverseControls {
        #[template_child]
        pub(super) run_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub(super) random_seed_button: TemplateChild<gtk::Button>,

        pub(super) playing: std::cell::Cell<bool>,
        pub(super) reveal_tools: std::cell::Cell<bool>,
        pub(super) brush_mode: std::cell::Cell<bool>,
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
                    ParamSpecBoolean::builder("playing")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("stopped")
                        .default_value(true)
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("reveal-tools")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("brush-mode")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecString::builder("run-button-icon-name")
                        .default_value(Some("media-playback-start-symbolic"))
                        .readwrite()
                        .build(),
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
                "reveal-tools" => imp.reveal_tools.get().to_value(),
                "brush-mode" => imp.brush_mode.get().to_value(),
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
                    let run_button = self.run_button.get();
                    let run_button_style = run_button.style_context();

                    if now_playing != was_playing {
                        self.playing.set(now_playing);

                        if now_playing {
                            run_button_style.remove_class("play");
                            run_button_style.add_class("stop");
                        } else {
                            run_button_style.remove_class("stop");
                            run_button_style.add_class("play");
                        }

                        obj.notify("run-button-icon-name");
                        obj.notify("playing");
                        obj.notify("stopped");
                    }
                }
                "reveal-tools" => {
                    obj.imp().reveal_tools.set(value.get::<bool>().unwrap());
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
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    pub fn set_tools_revealed(&self, value: bool) {
        self.imp().reveal_tools.set(value);
        self.notify("reveal-tools");
    }

    pub fn tools_revealed(&self) -> bool {
        self.imp().reveal_tools.get()
    }

    pub fn toggle_brush(&self) {
        self.imp().brush_mode.set(!self.brush());
        self.notify("brush-mode");
    }

    pub fn brush(&self) -> bool {
        self.imp().brush_mode.get()
    }
}
