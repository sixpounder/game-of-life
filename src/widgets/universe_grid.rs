use crate::config::G_LOG_DOMAIN;
use crate::lib::WindowedStack;
use crate::models::{
    Universe, UniverseCell, UniverseGridMode, UniversePoint, UniversePointMatrix, UniverseSnapshot,
};
use crate::services::GameOfLifeSettings;
use gtk::{
    gio,
    glib::clone,
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use async_channel::{Receiver, Sender};

use std::cell::{Cell, RefCell, Ref, RefMut};
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

        let universe_row = ((x.round() as i32) * universe_width as i32) / widget_width;
        let universe_column = ((y.round() as i32) * universe_height as i32) / widget_height;

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
    let animated = widget.animated.get();

    let mut outline_color = bg_color;
    outline_color.set_red(outline_color.red() + 0.1);
    outline_color.set_green(outline_color.green() + 0.1);
    outline_color.set_blue(outline_color.blue() + 0.1);

    // Paint the background
    snapshot.append_color(&bg_color, bounds);

    // Create a utility cairo context
    let cairo_context = snapshot.append_cairo(bounds);

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
            } else if animated {
                let transparency_factor = el.corpse_heat() as f32;
                if transparency_factor > 0.0 {
                    let cell_rect_bounds = gtk::graphene::Rect::new(
                        coords.0 as f32,
                        coords.1 as f32,
                        width as f32,
                        height as f32,
                    );
                    let mut fade_color = fg_color;
                    fade_color.set_alpha(fade_color.alpha() * transparency_factor);
                    snapshot.append_color(&fade_color, &cell_rect_bounds);
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
    use glib::{ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecUInt};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/sixpounder/GameOfLife/universe_grid.ui")]
    pub struct GameOfLifeUniverseGrid {
        /// Reference to the settings object
        pub(super) settings: GameOfLifeSettings,

        /// The universe being rendered
        pub(super) universe: RefCell<Option<Universe>>,

        /// A fixed size stack holding previous states
        pub(super) cache: RefCell<Option<WindowedStack<Universe>>>,

        /// Used to suspend or permit drawing operations
        pub(super) frozen: Cell<bool>,

        /// The speed at which universe states should tick
        pub(super) evolution_speed: Cell<u32>,

        /// Whether to animate cells transitions
        pub(super) animated: Cell<bool>,

        /// Whether to draw cells outlines
        pub(super) draw_cells_outline: Cell<bool>,

        pub(super) mode: Cell<UniverseGridMode>,

        pub(super) receiver: RefCell<Option<Receiver<UniverseGridRequest>>>,

        pub(super) sender: Option<Sender<UniverseGridRequest>>,

        pub(super) render_thread_stopper: RefCell<Option<std::sync::mpsc::Receiver<()>>>,

        pub(super) allow_draw_on_resize: Cell<bool>,

        /// The grid foreground color
        pub(super) fg_color: Cell<Option<gtk::gdk::RGBA>>,

        /// The grid background color
        pub(super) bg_color: Cell<Option<gtk::gdk::RGBA>>,

        /// Tracks the state of the interaction in order to drive interaction controllers
        pub(super) interaction_state: Cell<UniverseGridInteractionState>,

        /// While editing the grid this is used as the "next state" for each cell that gets edited.
        /// This will be tipically set at the beginning of a user interaction and unset at the end of it
        pub(super) interaction_next_state: Cell<Option<UniverseCell>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameOfLifeUniverseGrid {
        const NAME: &'static str = "GameOfLifeUniverseGrid";
        type Type = super::GameOfLifeUniverseGrid;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            let (sender, r) = async_channel::unbounded();
            let receiver = RefCell::new(Some(r));

            let mut this = Self::default();

            this.receiver = receiver;
            this.sender = Some(sender);

            // Start with a random universe
            this.universe.replace(Some(Universe::new_random(
                this.settings.universe_width() as usize,
                this.settings.universe_height() as usize,
            )));

            // Start universe in locked mode, meaning no interactions
            // are possible
            this.mode.set(UniverseGridMode::Locked);

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
                    ParamSpecEnum::builder("mode")
                        .default_value(UniverseGridMode::Locked)
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("allow-render-on-resize")
                        .default_value(false)
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("draw-cells-outline")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("running")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("animated")
                        .default_value(true)
                        .readwrite()
                        .build(),
                    ParamSpecUInt::builder("evolution-speed")
                        .minimum(1)
                        .maximum(100)
                        .default_value(5)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("can-rewind-one")
                        .default_value(false)
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "allow-render-on-resize" => {
                    obj.set_allow_render_on_resize(value.get::<bool>().unwrap_or(true));
                }
                "mode" => {
                    obj.set_mode(value.get::<UniverseGridMode>().unwrap());
                }
                "draw-cells-outline" => {
                    obj.set_draw_cells_outline(value.get::<bool>().unwrap());
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
                "animated" => obj.animated().to_value(),
                "evolution-speed" => obj.evolution_speed().to_value(),
                "running" => obj.is_running().to_value(),
                "can-rewind-one" => obj.can_rewind_one().to_value(),
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
            snapshot_grid(self, snapshot, &widget_bounds);
        }
    }
}

glib::wrapper! {
    pub struct GameOfLifeUniverseGrid(ObjectSubclass<imp::GameOfLifeUniverseGrid>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GameOfLifeUniverseGrid {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_channel(&self) {
        let receiver = self.imp().receiver.borrow_mut().take().unwrap();
        glib::spawn_future_local(
            clone!(@strong self as this => async move {
                loop {
                    let action: UniverseGridRequest = receiver.recv().await.unwrap();
                    this.process_action(action);
                }
            })
        );
    }

    /// Initializes the inner drawing area with callbacks, controllers etc...
    fn setup_drawing_area(&self) {
        let drawing_area = self.imp().obj();

        let left_click_gesture_controller = gtk::GestureClick::new();
        left_click_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        left_click_gesture_controller.connect_pressed(
            clone!(@strong self as this => move |gesture, n_press, x, y| {
                // Detect the cell at the point of interaction
                let clicked_cell = widget_area_point_to_universe_cell(
                    &this,
                    this.universe().as_ref(),
                    x,
                    y,
                );

                // Memorize the next state for each cell touched by this interaction
                // as the inverse of the state of the cell under the current pointing
                // device
                this.imp().interaction_next_state.set(
                    clicked_cell.map(|p| !p.cell().clone())
                );

                this.on_drawing_area_clicked(
                    gesture,
                    n_press,
                    x,
                    y,
                    this.interaction_next_state()
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
        drawing_area.add_controller(left_click_gesture_controller);

        let left_drag_gesture_controller = gtk::GestureDrag::new();
        left_drag_gesture_controller.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        left_drag_gesture_controller.connect_begin(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_begin(gesture, events, this.interaction_next_state());
            }),
        );

        left_drag_gesture_controller.connect_update(
            clone!(@strong self as this => move |gesture, events| {
                this.on_drawing_area_drag_move(gesture, events, this.interaction_next_state())
            }),
        );
        drawing_area.add_controller(left_drag_gesture_controller);
    }

    fn process_action(&self, action: UniverseGridRequest) -> glib::ControlFlow {
        match action {
            UniverseGridRequest::Unfreeze => self.set_frozen(false),
            UniverseGridRequest::Redraw(new_universe_state) => {
                if let Some(new_universe_state) = new_universe_state {
                    self.imp().universe.replace(Some(new_universe_state));
                }
                self.redraw();
            }
        }

        glib::ControlFlow::Continue
    }

    fn on_drawing_area_clicked(
        &self,
        _gesture: &gtk::GestureClick,
        _n_press: i32,
        x: f64,
        y: f64,
        alter_state: Option<UniverseCell>,
    ) {
        if self.editable() {
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
        let imp = self.imp();
        imp
            .interaction_state
            .set(UniverseGridInteractionState::Idle);
        imp.interaction_next_state.set(None);
    }

    fn on_drawing_area_click_unpaired_released(
        &self,
        _gesture: &gtk::GestureClick,
        _x: f64,
        _y: f64,
        _button: u32,
        _events: Option<&gtk::gdk::EventSequence>,
    ) {
        let imp = self.imp();
        imp
            .interaction_state
            .set(UniverseGridInteractionState::Idle);
        imp.interaction_next_state.set(None);
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

    /// Alters the universe cell visually located at `x` and `y` coordinates. If `Some(value)`
    /// is provided it will be used as the new cell value, else the opposite value of the current
    /// one will be set
    fn alter_universe_point(&self, x: f64, y: f64, value: Option<UniverseCell>) {
        let universe_borrow = self.imp().universe.borrow();

        if let Some(universe_point) =
            widget_area_point_to_universe_cell(self, universe_borrow.as_ref(), x, y)
        {
            // If a point is found, set its cell value
            drop(universe_borrow);
            let mut universe_mut_borrow = self.universe_mut();
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
          self.notify("mode");
      }
    }

    pub fn editable(&self) -> bool {
        !self.is_running() && self.mode() != UniverseGridMode::Locked
    }

    pub fn is_running(&self) -> bool {
        self.imp().render_thread_stopper.borrow().is_some()
    }

    pub fn can_rewind_one(&self) -> bool {
        self.imp().cache.borrow().as_ref()
            .map(|c| !c.is_empty())
            .unwrap_or(false)
    }

    pub fn set_frozen(&self, value: bool) {
        self.imp().frozen.set(value);
        if !value {
            self.queue_draw();
        }
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
        let universe_action_sender = self.get_sender();

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
            glib::spawn_future(async move {
                while thread_render_stopper_sender.send(()).is_ok() {
                    glib::timeout_future(std::time::Duration::from_millis(wait)).await;
                    thread_universe.tick();
                    let _ = universe_action_sender.send(UniverseGridRequest::Redraw(Some(thread_universe.clone()))).await;
                }
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

    pub fn skip_forward_one(&self) {
        if let Ok(mut borrow) = self.imp().universe.try_borrow_mut() {
            if let Some(current_universe) = borrow.as_mut() {
                current_universe.tick();
                self.redraw();
            }
        }
    }

    pub fn rewind_one(&self) {
        if let Ok(mut borrow) = self.imp().universe.try_borrow_mut() {

        }
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

    pub fn universe(&self) -> Ref<Option<Universe>> {
        self.imp().universe.borrow()
    }

    pub fn universe_mut(&self) -> RefMut<Option<Universe>> {
        self.imp().universe.borrow_mut()
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

    fn interaction_next_state(&self) -> Option<UniverseCell> {
        self.imp().interaction_next_state.get()
    }
}


