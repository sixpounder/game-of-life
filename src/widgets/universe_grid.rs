use crate::config::{APPLICATION_ID, G_LOG_DOMAIN};
use crate::models::{
    Universe, UniverseGridMode, UniversePoint, UniversePointMatrix, UniverseSnapshot,
};
use gtk::{
    gio, glib,
    glib::{clone, Receiver, Sender},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::{Cell, RefCell};
use std::str::FromStr;

const FG_COLOR_LIGHT: &str = "#64baff";
const BG_COLOR_LIGHT: &str = "#fafafa";
const FG_COLOR_DARK: &str = "#C061CB";
const BG_COLOR_DARK: &str = "#3D3846";

/// Maps a point on the widget area onto a cell in a given universe
fn widget_area_point_to_universe_cell(
    drawing_area: &gtk::DrawingArea,
    universe: Option<&Universe>,
    x: f64,
    y: f64,
) -> Option<UniversePoint> {
    if let Some(universe) = universe {
        let (widget_width, widget_height) = (drawing_area.width(), drawing_area.height());
        let (universe_width, universe_height) = (universe.rows(), universe.columns());

        let universe_row = ((x.round() as i32) * universe_width as i32) / widget_width as i32;
        let universe_column = ((y.round() as i32) * universe_height as i32) / widget_height as i32;

        Some(universe.get(universe_row as usize, universe_column as usize))
    } else {
        None
    }
}

#[derive(Debug)]
pub enum UniverseGridRequest {
        /// Freezes rendering process. Useful when moving windows and the likes.
    Freeze,

    /// Restores normal rendering operations
    Unfreeze,

    /// Sets the widget mode (run or design)
    Mode(UniverseGridMode),

    /// Sets the color scheme for the grid
    DarkColorSchemePreference(bool),

    /// Starts evolving the inner universe
    Run,

    /// Halts the evolution of the inner universe
    Halt,

    /// Requests the grid to redraw itself.If the value is Some(universe) the contained
    /// value will replace the current model inside the widget
    Redraw(Option<Universe>),
}

mod imp {
    use super::*;
    use glib::{
        types::StaticType, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject,
    };
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_grid.ui")]
    pub struct GameOfLifeUniverseGrid {
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,

        pub(crate) mode: Cell<UniverseGridMode>,

        pub(crate) frozen: Cell<bool>,

        pub(crate) prefers_dark_mode: Cell<bool>,

        pub(crate) universe: RefCell<Option<Universe>>,

        pub(crate) receiver: RefCell<Option<Receiver<UniverseGridRequest>>>,

        pub(crate) sender: Option<Sender<UniverseGridRequest>>,

        pub(crate) render_thread_stopper: RefCell<Option<std::sync::mpsc::Receiver<()>>>,

        pub(crate) fg_color: std::cell::Cell<Option<gtk::gdk::RGBA>>,

        pub(crate) bg_color: std::cell::Cell<Option<gtk::gdk::RGBA>>,
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

            this.universe.replace(Some(Universe::new_random(200, 200)));

            this.receiver = receiver;
            this.sender = Some(sender);
            this.mode.set(UniverseGridMode::Run);

            // Defaults to light color scheme
            this.fg_color
                .set(Some(gtk::gdk::RGBA::from_str(FG_COLOR_LIGHT).unwrap()));
            this.bg_color
                .set(Some(gtk::gdk::RGBA::from_str(BG_COLOR_DARK).unwrap()));

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
                        1,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("frozen", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("is-running", "", "", false, ParamFlags::READABLE),
                    ParamSpecBoolean::new(
                        "prefers-dark-mode",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE,
                    ),
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
                "prefers-dark-mode" => {
                    obj.imp()
                        .prefers_dark_mode
                        .replace(value.get::<bool>().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "mode" => self.mode.get().to_value(),
                "frozen" => self.frozen.get().to_value(),
                "prefers-dark-mode" => self.prefers_dark_mode.get().to_value(),
                "is-running" => obj.is_running().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GameOfLifeUniverseGrid {}
}

glib::wrapper! {
    pub struct GameOfLifeUniverseGrid(ObjectSubclass<imp::GameOfLifeUniverseGrid>)
        @extends gtk::Widget,
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

    /// Initializes the inner drawing area with callbacks, controllers etc...
    fn setup_drawing_area(&self) {
        let drawing_area = self.imp().drawing_area.get();

        let controller = gtk::GestureClick::new();
        controller.connect_pressed(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                this.on_drawing_area_clicked(gesture, n_press, x, y);
            }),
        );
        drawing_area.add_controller(&controller);

        drawing_area.connect_resize(
            clone!(@strong self as this => move |_widget, _width, _height| {
                this.set_frozen(true);
                let sender = this.get_sender();
                glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                    sender.send(UniverseGridRequest::Unfreeze).expect("Could not unlock grid");
                });
            }),
        );

        drawing_area.set_draw_func(
            clone!(@strong self as this => move |widget, context, width, height| this.render(widget, context, width, height) ),
        );
    }

    fn process_action(&self, action: UniverseGridRequest) -> glib::Continue {
        match action {
            UniverseGridRequest::Freeze => self.set_frozen(true),
            UniverseGridRequest::Unfreeze => self.set_frozen(false),
            UniverseGridRequest::Mode(m) => self.set_mode(m),
            UniverseGridRequest::Run => self.run(),
            UniverseGridRequest::Halt => self.halt(),
            UniverseGridRequest::Redraw(new_universe_state) => {
                if let Some(new_universe_state) = new_universe_state {
                    self.imp().universe.replace(Some(new_universe_state));
                }
                self.redraw();
            }
            UniverseGridRequest::DarkColorSchemePreference(prefers_dark) => {
                self.set_prefers_dark_mode(prefers_dark)
            }
        }

        glib::Continue(true)
    }

    fn on_drawing_area_clicked(&self, _gesture: &gtk::GestureClick, _n_press: i32, x: f64, y: f64) {
        if self.mode() == UniverseGridMode::Design {
            let universe_borrow = self.imp().universe.borrow();
            if let Some(universe_point) = widget_area_point_to_universe_cell(
                &self.imp().drawing_area,
                universe_borrow.as_ref(),
                x,
                y,
            ) {
                // If a point is found, invert its cell value
                drop(universe_borrow);
                let mut universe_mut_borrow = self.imp().universe.borrow_mut();
                let mut_borrow = universe_mut_borrow.as_mut().unwrap();
                mut_borrow.set_cell(
                    universe_point.row(),
                    universe_point.column(),
                    !(*universe_point.cell()),
                );
                self.redraw();
            }
        }
    }

    pub fn set_prefers_dark_mode(&self, prefers_dark_variant: bool) {
        let imp = self.imp();
        imp.prefers_dark_mode.replace(prefers_dark_variant);

        match prefers_dark_variant {
            true => {
                imp.fg_color
                    .set(Some(gtk::gdk::RGBA::from_str(FG_COLOR_DARK).unwrap()));
                imp.bg_color
                    .set(Some(gtk::gdk::RGBA::from_str(BG_COLOR_DARK).unwrap()));
            }
            false => {
                imp.fg_color
                    .set(Some(gtk::gdk::RGBA::from_str(FG_COLOR_LIGHT).unwrap()));
                imp.bg_color
                    .set(Some(gtk::gdk::RGBA::from_str(BG_COLOR_LIGHT).unwrap()));
            }
        }
    }

    pub fn prefers_dark_mode(&self) -> bool {
        self.imp().prefers_dark_mode.get()
    }

    fn render(
        &self,
        _widget: &gtk::DrawingArea,
        context: &gtk::cairo::Context,
        width: i32,
        height: i32,
    ) {
        if !self.frozen() {
            let imp = self.imp();

            // Determine colors
            let fg_color = imp.fg_color.get().unwrap();
            let bg_color = imp.bg_color.get().unwrap();

            context.set_source_rgba(
                bg_color.red() as f64,
                bg_color.green() as f64,
                bg_color.blue() as f64,
                bg_color.alpha() as f64,
            );
            context.rectangle(0.0, 0.0, width.into(), height.into());
            context.fill().unwrap();

            // Get a lock on the universe object
            let universe = self.imp().universe.borrow();
            if let Some(universe) = universe.as_ref() {
                let (width, height) = (
                    width as f64 / universe.columns() as f64,
                    height as f64 / universe.rows() as f64,
                );

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
                        let coords: (f64, f64) = ((w as f64) * width, (h as f64) * height);

                        context.rectangle(coords.0, coords.1, width, height);
                        context.fill().unwrap();
                    }
                }
            } else {
                glib::warn!("No universe to render");
            }
        }
    }

    pub fn mode(&self) -> UniverseGridMode {
        self.imp().mode.get()
    }

    pub fn set_mode(&self, value: UniverseGridMode) {
        if !self.is_running() {
            self.imp().mode.set(value);

            match self.mode() {
                UniverseGridMode::Design => {}
                UniverseGridMode::Run => {}
            }
        }

        self.notify("mode");
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
        self.set_mode(UniverseGridMode::Run);
        let local_sender = self.get_sender();

        let (thread_render_stopper_sender, thread_render_stopper_receiver) =
            std::sync::mpsc::channel::<()>();

        // Drop this to stop ticking thread
        self.imp()
            .render_thread_stopper
            .replace(Some(thread_render_stopper_receiver));

        let thread_universe = self.imp().universe.borrow();
        if let Some(universe) = thread_universe.as_ref() {
            let mut thread_universe = universe.clone();
            std::thread::spawn(move || loop {
                match thread_render_stopper_sender.send(()) {
                    Ok(_) => (),
                    Err(_) => break,
                };

                std::thread::sleep(std::time::Duration::from_millis(50));
                thread_universe.tick();
                local_sender
                    .send(UniverseGridRequest::Redraw(Some(thread_universe.clone())))
                    .unwrap();
            });

            self.notify("is-running");
        } else {
            glib::warn!("No universe to run");
        }
    }

    pub fn halt(&self) {
        let inner = self.imp().render_thread_stopper.take();
        drop(inner);
        self.notify("is-running");
    }

    pub fn get_universe_snapshot(&self) -> UniverseSnapshot {
        let imp = self.imp();
        imp.universe.borrow().as_ref().unwrap().snapshot()
    }

    pub fn random_seed(&self) {
        let current_universe = self.imp().universe.borrow();
        let (rows, cols) = match current_universe.as_ref() {
            Some(universe) => (universe.rows(), universe.columns()),
            None => (200, 200),
        };

        drop(current_universe);

        let new_universe = Universe::new_random(rows, cols);
        self.process_action(UniverseGridRequest::Redraw(Some(new_universe)));
    }

    pub fn redraw(&self) {
        self.imp().drawing_area.queue_draw();
    }
}


