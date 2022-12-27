use crate::config::G_LOG_DOMAIN;
use crate::models::{
    Universe, UniverseCell, UniverseGridMode, UniversePoint, UniversePointMatrix, UniverseSnapshot,
};
use crate::services::GameOfLifeSettings;
use gtk::{
    gio,
    glib::{clone, Receiver, Sender},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::{Cell, RefCell};
use std::str::FromStr;

/// Maps a point on the widget area onto a cell in a given universe
fn widget_area_point_to_universe_cell(
    drawing_area: &GameOfLifeUniverseGrid,
    universe: Option<&Universe>,
    x: f64,
    y: f64,
) -> Option<UniversePoint> {
    if let Some(universe) = universe {
        let (widget_width, widget_height) = (drawing_area.width(), drawing_area.height());
        let (universe_width, universe_height) = (universe.rows(), universe.columns());

        let universe_row = ((x.round() as i32) * universe_width as i32) / widget_width as i32;
        let universe_column = ((y.round() as i32) * universe_height as i32) / widget_height as i32;

        universe.get(universe_row as usize, universe_column as usize)
    } else {
        None
    }
}

fn snapshot_grid(
    widget: &imp::GameOfLifeUniverseGrid,
    snapshot: &gtk::Snapshot,
    bounds: &gtk::graphene::Rect,
) {
    // Determine colors
    let fg_color = widget.fg_color.get().unwrap();
    let bg_color = widget.bg_color.get().unwrap();
    let wants_outlines = widget.draw_cells_outline.get();
    let fades_dead_cells = widget.fades_dead_cells.get();

    let mut outline_color = bg_color.clone();
    outline_color.set_red(outline_color.red() + 0.1);
    outline_color.set_green(outline_color.green() + 0.1);
    outline_color.set_blue(outline_color.blue() + 0.1);

    // Paint the background
    snapshot.append_color(&bg_color, &bounds);

    // Create a utility cairo context
    let cairo_context = snapshot.append_cairo(&bounds);

    // Get a lock on the universe object
    let universe = widget.universe.borrow();
    if let Some(universe) = universe.as_ref() {
        let (width, height) = (
            bounds.width() as f64 / universe.columns() as f64,
            bounds.height() as f64 / universe.rows() as f64,
        );

        for el in universe.iter_cells() {
            let w = el.row();
            let h = el.column();
            let coords: (f64, f64) = ((w as f64) * width, (h as f64) * height);

            if wants_outlines {
                cairo_context.rectangle(coords.0, coords.1, width, height);
                cairo_context.set_line_width(1.0);
                cairo_context.set_source_rgba(
                    outline_color.red() as f64,
                    outline_color.green() as f64,
                    outline_color.blue() as f64,
                    outline_color.alpha() as f64,
                );
                cairo_context.stroke().unwrap();
            }
            if el.cell().is_alive() {
                let cell_rect_bounds = gtk::graphene::Rect::new(
                    coords.0 as f32,
                    coords.1 as f32,
                    width as f32,
                    height as f32,
                );
                snapshot.append_color(&fg_color, &cell_rect_bounds);
            } else {
                if fades_dead_cells {
                    let transparency_factor = el.corpse_heat();
                    if transparency_factor > 0.0 {
                        let cell_rect_bounds = gtk::graphene::Rect::new(
                            coords.0 as f32,
                            coords.1 as f32,
                            width as f32,
                            height as f32,
                        );
                        let mut fade_color = fg_color.clone();
                        fade_color.set_alpha(fade_color.alpha() * transparency_factor as f32);
                        snapshot.append_color(&fade_color, &cell_rect_bounds);
                    }
                }
            }
        }
    } else {
        glib::warn!("No universe to render");
    }
}

#[derive(Debug)]
pub enum UniverseGridRequest {
    /// Restores normal rendering operations
    #[allow(dead_code)]
    Unfreeze,

    /// Requests the grid to redraw itself. If the value is Some(universe) the contained
    /// value will replace the current model inside the widget
    Redraw(Option<Universe>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum UniverseGridInteractionState {
    Idle,
    Ongoing,
}

impl Default for UniverseGridInteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

mod imp {
    use super::*;
    use glib::{
        types::StaticType, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecUInt,
    };
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_grid.ui")]
    pub struct GameOfLifeUniverseGrid {
        pub(super) settings: GameOfLifeSettings,

        pub(super) frozen: Cell<bool>,

        pub(super) mode: Cell<UniverseGridMode>,

        pub(super) universe: RefCell<Option<Universe>>,

        pub(super) receiver: RefCell<Option<Receiver<UniverseGridRequest>>>,

        pub(super) sender: Option<Sender<UniverseGridRequest>>,

        pub(super) render_thread_stopper: RefCell<Option<std::sync::mpsc::Receiver<()>>>,

        pub(super) allow_draw_on_resize: Cell<bool>,

        pub(super) fg_color: Cell<Option<gtk::gdk::RGBA>>,

        pub(super) bg_color: Cell<Option<gtk::gdk::RGBA>>,

        pub(super) point_under_pointing_device: Cell<Option<UniversePoint>>,

        pub(super) evolution_speed: Cell<u32>,

        pub(super) animated: Cell<bool>,

        pub(super) draw_cells_outline: Cell<bool>,

        pub(super) fades_dead_cells: Cell<bool>,

        pub(super) interaction_state: Cell<UniverseGridInteractionState>,
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

            this.universe.replace(Some(Universe::new_random(
                this.settings.universe_width() as usize,
                this.settings.universe_height() as usize,
            )));

            this.receiver = receiver;
            this.sender = Some(sender);

            // Start universe in running mode
            this.mode.set(UniverseGridMode::Run);

            // Defaults to light color scheme
            this.fg_color.set(Some(
                gtk::gdk::RGBA::from_str(&this.settings.fg_color()).unwrap(),
            ));

            this.bg_color.set(Some(
                gtk::gdk::RGBA::from_str(&this.settings.bg_color()).unwrap(),
            ));

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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
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
                    ParamSpecBoolean::new(
                        "allow-render-on-resize",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "draw-cells-outline",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("running", "", "", false, ParamFlags::READABLE),
                    ParamSpecBoolean::new("animated", "", "", true, ParamFlags::READWRITE),
                    ParamSpecUInt::new("evolution-speed", "", "", 1, 100, 5, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "allow-render-on-resize" => {
                    obj.set_allow_render_on_resize(value.get::<bool>().unwrap());
                }
                "mode" => {
                    obj.set_mode(value.get::<UniverseGridMode>().unwrap());
                }
                "draw-cells-outline" => {
                    obj.set_draw_cells_outline(value.get::<bool>().unwrap());
                }
                "fades-dead-cells" => {
                    obj.set_fades_dead_cells(value.get::<bool>().unwrap());
                }
                "animated" => {
                    obj.set_animated(value.get::<bool>().unwrap());
                }
                "evolution-speed" => {
                    obj.set_evolution_speed(value.get::<u32>().unwrap_or(5));
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "mode" => self.mode.get().to_value(),
                "allow-render-on-resize" => self.allow_draw_on_resize.get().to_value(),
                "draw-cells-outline" => obj.draw_cells_outline().to_value(),
                "fades-dead-cells" => obj.fades_dead_cells().to_value(),
                "animated" => obj.animated().to_value(),
                "evolution-speed" => obj.evolution_speed().to_value(),
                "running" => obj.is_running().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GameOfLifeUniverseGrid {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget_bounds = gtk::graphene::Rect::new(
                0.0,
                0.0,
                self.obj().width() as f32,
                self.obj().height() as f32,
            );
            snapshot_grid(&self, &snapshot, &widget_bounds);
        }
    }
}

glib::wrapper! {
    pub struct GameOfLifeUniverseGrid(ObjectSubclass<imp::GameOfLifeUniverseGrid>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeUniverseGrid {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new::<Self>(&[("application", application)])
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
        let drawing_area = self.imp().obj();

        let left_click_gesture_controller = gtk::GestureClick::new();
        left_click_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        left_click_gesture_controller.connect_pressed(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                this.on_drawing_area_clicked(
                    gesture,
                    n_press,
                    x,
                    y,
                    Some(UniverseCell::Alive)
                );
            }),
        );
        left_click_gesture_controller.connect_released(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                this.on_drawing_area_click_released(gesture, n_press, x, y);
            }),
        );
        left_click_gesture_controller.connect_unpaired_release(
            clone!(@strong self as this => move |gesture, x, y, button, events| {
                this.on_drawing_area_click_unpaired_released(gesture, x, y, button, events);
            }),
        );
        drawing_area.add_controller(&left_click_gesture_controller);

        let right_click_gesture_controller = gtk::GestureClick::new();
        right_click_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        right_click_gesture_controller.connect_pressed(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                this.on_drawing_area_clicked(
                    gesture,
                    n_press,
                    x,
                    y,
                    Some(UniverseCell::Dead)
                );
            }),
        );
        right_click_gesture_controller.connect_released(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                this.on_drawing_area_click_released(gesture, n_press, x, y);
            }),
        );
        right_click_gesture_controller.connect_unpaired_release(
            clone!(@strong self as this => move |gesture, x, y, button, events| {
                this.on_drawing_area_click_unpaired_released(gesture, x, y, button, events);
            }),
        );
        drawing_area.add_controller(&right_click_gesture_controller);

        let left_drag_gesture_controller = gtk::GestureDrag::new();
        left_drag_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        left_drag_gesture_controller.connect_begin(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_begin(gesture, events, Some(UniverseCell::Alive))
            }),
        );

        left_drag_gesture_controller.connect_update(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_move(gesture, events, Some(UniverseCell::Alive))
            }),
        );
        drawing_area.add_controller(&left_drag_gesture_controller);

        let right_drag_gesture_controller = gtk::GestureDrag::new();
        right_drag_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        right_drag_gesture_controller.connect_begin(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_begin(gesture, events, Some(UniverseCell::Dead))
            }),
        );

        right_drag_gesture_controller.connect_update(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_move(gesture, events, Some(UniverseCell::Dead))
            }),
        );
        drawing_area.add_controller(&right_drag_gesture_controller);

        let motion_controller = gtk::EventControllerMotion::new();
        motion_controller.connect_enter(clone!(@strong self as this => move |controller, x, y| {
            this.on_drawing_area_mouse_position(controller, x, y);
        }));

        motion_controller.connect_leave(clone!(@strong self as this => move |controller| {
            this.on_drawing_area_mouse_leave(controller);
        }));

        motion_controller.connect_motion(clone!(@strong self as this => move |controller, x, y| {
            this.on_drawing_area_mouse_position(controller, x, y);
        }));

        drawing_area.add_controller(&motion_controller);
    }

    fn process_action(&self, action: UniverseGridRequest) -> glib::Continue {
        match action {
            UniverseGridRequest::Unfreeze => self.set_frozen(false),
            UniverseGridRequest::Redraw(new_universe_state) => {
                if let Some(new_universe_state) = new_universe_state {
                    self.imp().universe.replace(Some(new_universe_state));
                }
                self.redraw();
            }
        }

        glib::Continue(true)
    }

    fn on_drawing_area_clicked(
        &self,
        _gesture: &gtk::GestureClick,
        _n_press: i32,
        x: f64,
        y: f64,
        alter_state: Option<UniverseCell>,
    ) {
        if self.mode() == UniverseGridMode::Design {
            self.imp()
                .interaction_state
                .set(UniverseGridInteractionState::Ongoing);
            self.alter_universe_point(x, y, alter_state);
        }
    }

    fn on_drawing_area_click_released(
        &self,
        _gesture: &gtk::GestureClick,
        _n_press: i32,
        _x: f64,
        _y: f64,
    ) {
        self.imp()
            .interaction_state
            .set(UniverseGridInteractionState::Idle);
    }

    fn on_drawing_area_click_unpaired_released(
        &self,
        _gesture: &gtk::GestureClick,
        _x: f64,
        _y: f64,
        _button: u32,
        _events: Option<&gtk::gdk::EventSequence>,
    ) {
        self.imp()
            .interaction_state
            .set(UniverseGridInteractionState::Idle);
    }

    fn on_drawing_area_drag_begin(
        &self,
        gesture: &gtk::GestureDrag,
        _events: Option<&gtk::gdk::EventSequence>,
        alter_state: Option<UniverseCell>,
    ) {
        if self.imp().interaction_state.get() == UniverseGridInteractionState::Ongoing {
            if let Some(point) = gesture.start_point() {
                self.alter_universe_point(point.0, point.1, alter_state);
            }
        }
    }

    fn on_drawing_area_drag_move(
        &self,
        gesture: &gtk::GestureDrag,
        _events: Option<&gtk::gdk::EventSequence>,
        alter_state: Option<UniverseCell>,
    ) {
        if self.imp().interaction_state.get() == UniverseGridInteractionState::Ongoing {
            if let Some(point) = gesture.offset() {
                let origin = gesture.start_point().unwrap();
                self.alter_universe_point(origin.0 + point.0, origin.1 + point.1, alter_state);
            }
        }
    }

    fn on_drawing_area_mouse_position(
        &self,
        _controller: &gtk::EventControllerMotion,
        x: f64,
        y: f64,
    ) {
        self.imp()
            .point_under_pointing_device
            .set(widget_area_point_to_universe_cell(
                &self.imp().obj(),
                self.imp().universe.borrow().as_ref(),
                x,
                y,
            ));
    }

    fn on_drawing_area_mouse_leave(&self, _controller: &gtk::EventControllerMotion) {
        self.imp().point_under_pointing_device.set(None);
    }

    /// Alters the universe cell visually located at `x` and `y` coordinates. If `Some(value)`
    /// is provided it will be used as the new cell value, else the opposite value of the current
    /// one will be set
    fn alter_universe_point(&self, x: f64, y: f64, value: Option<UniverseCell>) {
        let drawing_area = self.imp().obj();
        let universe_borrow = self.imp().universe.borrow();

        if let Some(universe_point) =
            widget_area_point_to_universe_cell(&drawing_area, universe_borrow.as_ref(), x, y)
        {
            // If a point is found, set its cell value
            drop(universe_borrow);
            let mut universe_mut_borrow = self.imp().universe.borrow_mut();
            let mut_borrow = universe_mut_borrow.as_mut().unwrap();

            // NONE value means invert the cell value, SOME value sets it
            let next_value = match value {
                Some(v) => v,
                None => !(*universe_point.cell()),
            };

            mut_borrow.set_cell(universe_point.row(), universe_point.column(), next_value);
            self.redraw();
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
                self.queue_draw();
            }
            _ => (),
        }

        self.imp().frozen.set(value);
    }

    pub fn frozen(&self) -> bool {
        self.imp().frozen.get()
    }

    pub fn allow_draw_on_resize(&self) -> bool {
        self.imp().allow_draw_on_resize.get()
    }

    pub fn set_allow_render_on_resize(&self, value: bool) {
        self.imp().allow_draw_on_resize.set(value);
    }

    fn get_sender(&self) -> Sender<UniverseGridRequest> {
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
            let wait: u64 = 1000 / u64::from(self.evolution_speed());
            std::thread::spawn(move || loop {
                match thread_render_stopper_sender.send(()) {
                    Ok(_) => (),
                    Err(_) => break,
                };

                std::thread::sleep(std::time::Duration::from_millis(wait));
                thread_universe.tick();
                local_sender
                    .send(UniverseGridRequest::Redraw(Some(thread_universe.clone())))
                    .unwrap();
            });

            self.notify("running");
        } else {
            glib::warn!("No universe to run");
        }
    }

    pub fn halt(&self) {
        let inner = self.imp().render_thread_stopper.take();
        drop(inner);
        self.notify("running");
    }

    pub fn toggle_run(&self) {
        if self.is_running() {
            self.halt();
        } else {
            self.run();
        }
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

    pub fn set_universe(&self, universe: Universe) {
        self.imp().universe.replace(Some(universe));
        self.redraw();
    }

    pub fn redraw(&self) {
        self.queue_draw();
    }

    pub fn set_cell_color(&self, color: Option<gtk::gdk::RGBA>) {
        self.imp().fg_color.set(color);
        self.redraw();
    }

    pub fn set_background_color(&self, color: Option<gtk::gdk::RGBA>) {
        self.imp().bg_color.set(color);
        self.redraw();
    }

    pub fn rows(&self) -> usize {
        self.imp().universe.borrow().as_ref().unwrap().rows()
    }

    pub fn columns(&self) -> usize {
        self.imp().universe.borrow().as_ref().unwrap().columns()
    }

    pub fn draw_cells_outline(&self) -> bool {
        self.imp().draw_cells_outline.get()
    }

    pub fn set_draw_cells_outline(&self, value: bool) {
        let current = self.imp().draw_cells_outline.get();
        if value != current {
            self.imp().draw_cells_outline.set(value);
            self.redraw();
        }
    }

    pub fn fades_dead_cells(&self) -> bool {
        self.imp().fades_dead_cells.get()
    }

    pub fn set_fades_dead_cells(&self, value: bool) {
        let current = self.imp().fades_dead_cells.get();
        if value != current {
            self.imp().fades_dead_cells.set(value);

            if !self.is_running() {
                self.redraw();
            }
        }
    }

    pub fn evolution_speed(&self) -> u32 {
        self.imp().evolution_speed.get()
    }

    pub fn set_evolution_speed(&self, value: u32) {
        self.imp().evolution_speed.set(value);
    }

    pub fn animated(&self) -> bool {
        self.imp().animated.get()
    }

    pub fn set_animated(&self, value: bool) {
        self.imp().animated.set(value);
    }
}
