use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, glib::clone, CompositeTemplate};
use crate::models::UniverseGridMode;
use crate::widgets::UniverseGridRequest;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/window.ui")]
    pub struct GameOfLifeWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,

        #[template_child]
        pub universe_grid: TemplateChild<crate::widgets::GameOfLifeUniverseGrid>,

        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,

        pub(crate) running: std::cell::Cell<bool>,

        pub(crate) mode: std::cell::Cell<UniverseGridMode>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeWindow {
        const NAME: &'static str = "GameOfLifeWindow";
        type Type = super::GameOfLifeWindow;
        type ParentType = gtk::ApplicationWindow;

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
            obj.imp().running.set(false);
            obj.connect_events();
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
        glib::Object::new(&[("application", application)])
            .expect("Failed to create GameOfLifeWindow")
    }

    pub fn mode(&self) -> UniverseGridMode {
        self.imp().mode.get()
    }

    pub fn set_mode(&self, value: UniverseGridMode) {
        self.imp().mode.set(value);
    }

    fn connect_events(&self) {
        self.imp().run_button.connect_clicked(
            clone!(@strong self as this => move |_widget| {
                this.toggle_run();
            })
        );
    }

    pub fn toggle_run(&self) {
        let currently_running = self.imp().running.get();
        self.imp().running.set(!currently_running);

        let sender = self.imp().universe_grid.get_sender();
        match currently_running {
            false => sender.send(UniverseGridRequest::Run).unwrap(),
            true => sender.send(UniverseGridRequest::Halt).unwrap()
        }
    }
}
