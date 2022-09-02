use crate::models::{Universe, UniverseGridMode};
use gtk::{
    gio, glib,
    glib::{clone, Receiver, Sender},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use std::cell::{Cell, RefCell};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum UniverseGridRequest {
    Lock,
    Unlock,
    Run,
    Halt,
    Redraw,
}

mod imp {
    use super::*;
    use glib::{
        types::StaticType, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum,
    };
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_grid.ui")]
    pub struct GameOfLifeUniverseGrid {
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,

        pub(crate) mode: Cell<UniverseGridMode>,

        pub(crate) frozen: Cell<bool>,

        pub(crate) universe: Arc<Mutex<Universe>>,

        pub(crate) receiver: RefCell<Option<Receiver<UniverseGridRequest>>>,

        pub(crate) sender: Option<Sender<UniverseGridRequest>>,

        pub(crate) render_thread_stopper: RefCell<Option<std::sync::mpsc::Receiver<()>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeUniverseGrid {
        const NAME: &'static str = "GameOfLifeUniverseGrid";
        type Type = super::GameOfLifeUniverseGrid;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            let mut this = Self::default();

            this.receiver = receiver;
            this.sender = Some(sender);

            this
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GameOfLifeUniverseGrid {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_drawing_area();
            obj.setup_channel();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "mode",
                        "",
                        "",
                        UniverseGridMode::static_type(),
                        0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("frozen", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("is-running", "", "", false, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "mode" => {
                    obj.set_mode(value.get::<UniverseGridMode>().unwrap());
                }
                "frozen" => {
                    obj.set_frozen(value.get::<bool>().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "mode" => self.mode.get().to_value(),
                "frozen" => self.frozen.get().to_value(),
                "is-running" => obj.is_running().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GameOfLifeUniverseGrid {}
}

glib::wrapper! {
    pub struct GameOfLifeUniverseGrid(ObjectSubclass<imp::GameOfLifeUniverseGrid>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeUniverseGrid {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
            .expect("Failed to create GameOfLifeUniverseGrid")
    }

    fn setup_channel(&self) {
        let receiver = self.imp().receiver.borrow_mut().take().unwrap();
        receiver.attach(
            None,
            clone!(@strong self as this => move |action| this.process_action(action)),
        );
    }

    fn setup_drawing_area(&self) {
        self.imp().drawing_area.connect_resize(
            clone!(@strong self as this => move |_widget, _width, _height| {
                this.set_frozen(true);
                let sender = this.get_sender();
                glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                    sender.send(UniverseGridRequest::Unlock).expect("Could not unlock grid");
                });
            }),
        );

        self.imp().drawing_area.set_draw_func(
            clone!(@strong self as this => move |widget, context, width, height| this.render(widget, context, width, height) ),
        );
    }

    fn process_action(&self, action: UniverseGridRequest) -> glib::Continue {
        match action {
            UniverseGridRequest::Lock => self.set_frozen(true),
            UniverseGridRequest::Unlock => self.set_frozen(false),
            UniverseGridRequest::Run => self.run(),
            UniverseGridRequest::Halt => self.halt(),
            UniverseGridRequest::Redraw => self.imp().drawing_area.queue_draw(),
        }

        glib::Continue(true)
    }

    fn render(
        &self,
        _widget: &gtk::DrawingArea,
        context: &gtk::cairo::Context,
        width: i32,
        height: i32,
    ) {
        if !self.frozen() {
            let fg_color = gtk::gdk::RGBA::from_str("#64baff").unwrap();
            let bg_color = gtk::gdk::RGBA::from_str("#fafafa").unwrap();
            let universe = self.imp().universe.lock().unwrap();

            context.set_source_rgba(
                bg_color.red() as f64,
                bg_color.green() as f64,
                bg_color.blue() as f64,
                bg_color.alpha() as f64,
            );
            context.rectangle(0.0, 0.0, width.into(), height.into());
            context.fill().unwrap();

            let mut size: (f64, f64) = (
                width as f64 / universe.columns() as f64,
                height as f64 / universe.rows() as f64,
            );

            if size.0 <= size.1 {
                size = (size.0, size.0);
            } else {
                size = (size.1, size.1);
            }

            context.set_source_rgba(
                fg_color.red() as f64,
                fg_color.green() as f64,
                fg_color.blue() as f64,
                fg_color.alpha() as f64,
            );

            for el in universe.iter_cells() {
                if el.cell().is_alive() {
                    let w = el.row();
                    let h = el.column();
                    let coords: (f64, f64) = ((w as f64) * size.0, (h as f64) * size.1);

                    context.rectangle(coords.0, coords.1, size.0, size.1);
                    context.fill().unwrap();
                }
            }
        }
    }

    pub fn mode(&self) -> UniverseGridMode {
        self.imp().mode.get()
    }

    pub fn set_mode(&self, value: UniverseGridMode) {
        self.imp().mode.set(value);
        self.notify("mode");

        match self.mode() {
            UniverseGridMode::Design => {}
            UniverseGridMode::Run => {}
        }
    }

    pub fn is_running(&self) -> bool {
        self.imp().render_thread_stopper.borrow().is_some()
    }

    pub fn set_frozen(&self, value: bool) {
        match value {
            false => {
                self.imp().drawing_area.queue_draw();
            }
            _ => (),
        }

        self.imp().frozen.set(value);
    }

    pub fn frozen(&self) -> bool {
        self.imp().frozen.get()
    }

    pub fn get_sender(&self) -> Sender<UniverseGridRequest> {
        self.imp().sender.as_ref().unwrap().clone()
    }

    pub fn run(&self) {
        let universe = self.imp().universe.clone();
        let local_sender = self.get_sender();
        let (thread_render_sentinel, thread_render_receiver) =
            glib::MainContext::channel::<()>(glib::PRIORITY_DEFAULT);

        let (thread_render_stopper_sender, thread_render_stopper_receiver) =
            std::sync::mpsc::channel::<()>();

        self.imp()
            .render_thread_stopper
            .replace(Some(thread_render_stopper_receiver));

        std::thread::spawn(move || loop {
            match thread_render_stopper_sender.send(()) {
                Ok(_) => (),
                Err(_) => break,
            };

            std::thread::sleep(std::time::Duration::from_millis(50));
            let mut locked_universe = universe.lock().unwrap();
            locked_universe.tick();
            thread_render_sentinel.send(()).unwrap();
        });

        thread_render_receiver.attach(
            None,
            clone!(@strong self as this => move |_| {
                local_sender.send(UniverseGridRequest::Redraw).unwrap();
                glib::Continue(true)
            }),
        );

        self.notify("is-running");
    }

    pub fn halt(&self) {
        let inner = self.imp().render_thread_stopper.take();
        drop(inner);
        self.notify("is-running");
    }
}


