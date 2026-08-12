//! Drag-and-drop state shared by item slots.
//!
//! Port of the Conrod `SlotManager` interaction logic to an event-driven,
//! renderer-independent form. The manager is owned by the UI layer, shared
//! with every [`Slot`](super::slot::Slot) through a `RefCell`, and driven by
//! events reported by the slots.
use iced::{Point, keyboard::Modifiers};
use vek::*;

/// Slots must be comparable and know the size of the image shown while being
/// dragged.
pub trait SumSlot: Sized + PartialEq + Copy + Send + 'static {
    /// Size of the dragged content; `None` falls back to the manager default.
    fn drag_size(&self) -> Option<[f32; 2]>;
}

/// Events produced by the drag manager.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event<S> {
    /// Dragged from one slot onto another.
    Dragged(S, S),
    /// Dragged to open space (dropped on the ground/window).
    Dropped(S),
    /// Right-clicked while dragging half a stack over open space.
    SplitDropped(S),
    /// Right-clicked while dragging half a stack onto another slot.
    SplitDragged(S, S),
    /// Right-clicked a slot while not dragging (use item).
    Used(S),
    /// {Ctrl,Shift}-clicked a slot (move stack / move single item).
    Request { slot: S, auto_quantity: bool },
}

/// How a slot should render as a result of manager state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    Selected,
    Dragging,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ManagerState<S> {
    Dragging(S, Option<u32>),
    Selected(S),
    Idle,
}

/// Manages selection, dragging and dropping between slots.
#[derive(Debug)]
pub struct DragManager<S> {
    state: ManagerState<S>,
    // Rebuilt every frame via `begin_frame` + `register_slot`.
    slots: Vec<(S, Aabr<i32>)>,
    events: Vec<Event<S>>,
    mouse_over_slot: Option<S>,
    // Size to display dragged content when the slot has no preferred size.
    drag_img_size: Vec2<f32>,
    // Si prefix settings.
    use_prefixes: bool,
    prefix_switch_point: u32,
}

impl<S> DragManager<S>
where
    S: SumSlot,
{
    pub fn new(drag_img_size: Vec2<f32>, use_prefixes: bool, prefix_switch_point: u32) -> Self {
        Self {
            state: ManagerState::Idle,
            slots: Vec::new(),
            events: Vec::new(),
            mouse_over_slot: None,
            drag_img_size,
            use_prefixes,
            prefix_switch_point,
        }
    }

    pub fn set_use_prefixes(&mut self, use_prefixes: bool) { self.use_prefixes = use_prefixes; }

    pub fn set_prefix_switch_point(&mut self, prefix_switch_point: u32) {
        self.prefix_switch_point = prefix_switch_point;
    }

    /// Whether the amount text should use SI prefixes.
    pub fn use_prefixes(&self) -> bool { self.use_prefixes }

    pub fn prefix_switch_point(&self) -> u32 { self.prefix_switch_point }

    /// Clears the per-frame slot registry; call before building the UI.
    pub fn begin_frame(&mut self) {
        self.slots.clear();
        self.mouse_over_slot = None;
    }

    /// Registers a slot with its bounds so hover and drop targets can be
    /// resolved. `cursor` is used to update `mouse_over_slot`.
    pub fn register_slot(&mut self, slot: S, bounds: Aabr<i32>, cursor: Point) {
        if let Some(entry) = self.slots.iter_mut().find(|(s, _)| *s == slot) {
            entry.1 = bounds;
        } else {
            self.slots.push((slot, bounds));
        }
        if bounds_contains(bounds, cursor) {
            self.mouse_over_slot = Some(slot);
        }
    }

    /// The slot currently under the mouse cursor (per the last registration).
    pub fn mouse_over_slot(&self) -> Option<S> { self.mouse_over_slot }

    /// Default size used for the dragged-content ghost image.
    pub fn drag_img_size(&self) -> Vec2<f32> { self.drag_img_size }

    /// Returns `Some(slot)` if a slot is selected.
    pub fn selected(&self) -> Option<S> {
        match self.state {
            ManagerState::Selected(slot) => Some(slot),
            _ => None,
        }
    }

    /// Returns the source slot (and dragged stack amount) while dragging.
    pub fn dragging(&self) -> Option<(S, Option<u32>)> {
        match self.state {
            ManagerState::Dragging(slot, amount) => Some((slot, amount)),
            _ => None,
        }
    }

    /// The interaction state for rendering a given slot.
    pub fn interaction(&self, slot: S) -> Interaction {
        match self.state {
            ManagerState::Selected(s) if s == slot => Interaction::Selected,
            ManagerState::Dragging(s, _) if s == slot => Interaction::Dragging,
            _ => Interaction::None,
        }
    }

    /// Emits a `Used` event and deselects the selected slot.
    pub fn use_selected(&mut self) {
        if let ManagerState::Selected(slot) = self.state {
            self.events.push(Event::Used(slot));
            self.state = ManagerState::Idle;
        }
    }

    /// Emits a `Dropped` event and deselects the selected slot.
    pub fn dropped_selected(&mut self) {
        if let ManagerState::Selected(slot) = self.state {
            self.events.push(Event::Dropped(slot));
            self.state = ManagerState::Idle;
        }
    }

    /// Selects a specific slot; if it has no content it will be deselected on
    /// the next `on_slot_changed` call.
    pub fn select(&mut self, slot: S) { self.state = ManagerState::Selected(slot); }

    /// Sets the manager into an idle state.
    pub fn idle(&mut self) { self.state = ManagerState::Idle; }

    /// Called when a slot's content changes; cancels selection/dragging of
    /// slots that became empty.
    pub fn on_slot_changed(&mut self, slot: S, filled: bool) {
        if !filled {
            match self.state {
                ManagerState::Selected(s) | ManagerState::Dragging(s, _) if s == slot => {
                    self.state = ManagerState::Idle;
                },
                _ => {},
            }
        }
    }

    /// Left-click on a slot (port of the Conrod click/select/swap logic).
    pub fn on_click(&mut self, slot: S, filled: bool, click_count: u32, modifiers: Modifiers) {
        // Translate ctrl-clicks to stack-requests and shift-clicks to
        // individual-requests.
        if click_count > 0 && !matches!(self.state, ManagerState::Dragging(_, _)) {
            if modifiers.control {
                self.events.push(Event::Request {
                    slot,
                    auto_quantity: true,
                });
                self.state = ManagerState::Idle;
                return;
            } else if modifiers.shift {
                self.events.push(Event::Request {
                    slot,
                    auto_quantity: false,
                });
                self.state = ManagerState::Idle;
                return;
            }
        }

        if click_count > 0 {
            self.state = match self.state {
                ManagerState::Selected(other) => {
                    if slot != other {
                        self.events.push(Event::Dragged(other, slot));
                    }
                    if click_count == 1 {
                        // Clicked widget was already selected; deselect.
                        ManagerState::Idle
                    } else {
                        ManagerState::Selected(slot)
                    }
                },
                _ => {
                    // No widgets were selected.
                    if filled {
                        ManagerState::Selected(slot)
                    } else {
                        // Selected and then deselected with one or more clicks.
                        ManagerState::Idle
                    }
                },
            };
        }
    }

    /// Start dragging the contents of a slot.
    pub fn on_drag_start(&mut self, slot: S, amount: Option<u32>) {
        if !matches!(self.state, ManagerState::Dragging(_, _)) {
            self.state = ManagerState::Dragging(slot, amount);
        }
    }

    /// Left-button release while dragging; resolves the drop target from the
    /// registered slot bounds.
    pub fn on_release(&mut self, cursor: Point) {
        if let ManagerState::Dragging(from, _) = self.state {
            let target = self
                .slots
                .iter()
                .find(|(_, bounds)| bounds_contains(*bounds, cursor))
                .map(|(slot, _)| *slot);
            match target {
                Some(to) if to != from => self.events.push(Event::Dragged(from, to)),
                Some(_) | None => self.events.push(Event::Dropped(from)),
            }
            self.state = ManagerState::Idle;
        }
    }

    /// Right-click (port of the Conrod use/split logic).
    ///
    /// While dragging a partial stack this splits the stack; otherwise it
    /// emits a `Used` event for the slot under the cursor.
    pub fn on_right_click(&mut self, slot: S, cursor: Point) {
        match self.state {
            ManagerState::Dragging(from, Some(_)) => {
                let target = self
                    .slots
                    .iter()
                    .find(|(_, bounds)| bounds_contains(*bounds, cursor))
                    .map(|(slot, _)| *slot);
                match target {
                    Some(to) if to != from => self.events.push(Event::SplitDragged(from, to)),
                    Some(_) | None => self.events.push(Event::SplitDropped(from)),
                }
            },
            ManagerState::Dragging(_, None) => {},
            ManagerState::Selected(_) | ManagerState::Idle => {
                self.events.push(Event::Used(slot));
                // If something is selected, deselect.
                self.state = ManagerState::Idle;
            },
        }
    }

    /// Takes the events accumulated since the last call.
    pub fn take_events(&mut self) -> Vec<Event<S>> { core::mem::take(&mut self.events) }
}

fn bounds_contains(bounds: Aabr<i32>, cursor: Point) -> bool {
    let x = cursor.x.trunc() as i32;
    let y = cursor.y.trunc() as i32;
    x >= bounds.min.x && x < bounds.max.x && y >= bounds.min.y && y < bounds.max.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Slot(u8);

    impl SumSlot for Slot {
        fn drag_size(&self) -> Option<[f32; 2]> { None }
    }

    fn aabr(x: f32, y: f32, w: f32, h: f32) -> Aabr<i32> {
        Aabr {
            min: Vec2::new(x as i32, y as i32),
            max: Vec2::new((x + w) as i32, (y + h) as i32),
        }
    }

    fn cursor(x: f32, y: f32) -> Point { Point { x, y } }

    fn manager() -> DragManager<Slot> { DragManager::new(Vec2::new(40.0, 40.0), true, 6) }

    fn no_mods() -> Modifiers { Modifiers::default() }

    fn ctrl() -> Modifiers {
        Modifiers {
            control: true,
            ..Default::default()
        }
    }

    #[test]
    fn click_selects_filled_slot() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        assert_eq!(m.selected(), Some(Slot(0)));
        assert!(m.take_events().is_empty());
    }

    #[test]
    fn click_does_not_select_empty_slot() {
        let mut m = manager();
        m.on_click(Slot(0), false, 1, no_mods());
        assert_eq!(m.selected(), None);
    }

    #[test]
    fn click_selected_slot_deselects() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        m.on_click(Slot(0), true, 1, no_mods());
        assert_eq!(m.selected(), None);
    }

    #[test]
    fn click_other_selected_slot_swaps() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        m.on_click(Slot(1), true, 1, no_mods());
        assert_eq!(m.selected(), None);
        assert_eq!(m.take_events(), vec![Event::Dragged(Slot(0), Slot(1))]);
    }

    #[test]
    fn double_click_selects_after_swap() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        m.on_click(Slot(1), true, 2, no_mods());
        assert_eq!(m.selected(), Some(Slot(1)));
        assert_eq!(m.take_events(), vec![Event::Dragged(Slot(0), Slot(1))]);
    }

    #[test]
    fn ctrl_click_requests_stack() {
        let mut m = manager();
        m.on_click(Slot(2), true, 1, ctrl());
        assert_eq!(m.take_events(), vec![Event::Request {
            slot: Slot(2),
            auto_quantity: true,
        }]);
        assert_eq!(m.selected(), None);
    }

    #[test]
    fn right_click_uses_item() {
        let mut m = manager();
        m.on_right_click(Slot(3), cursor(5.0, 5.0));
        assert_eq!(m.take_events(), vec![Event::Used(Slot(3))]);
    }

    #[test]
    fn drag_drop_on_other_slot() {
        let mut m = manager();
        m.begin_frame();
        m.register_slot(Slot(0), aabr(0.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.register_slot(Slot(1), aabr(50.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.on_click(Slot(0), true, 1, no_mods());
        m.on_drag_start(Slot(0), Some(10));
        assert_eq!(m.dragging(), Some((Slot(0), Some(10))));
        m.on_release(cursor(60.0, 20.0));
        assert_eq!(m.take_events(), vec![Event::Dragged(Slot(0), Slot(1))]);
        assert_eq!(m.dragging(), None);
    }

    #[test]
    fn drag_drop_on_open_space() {
        let mut m = manager();
        m.begin_frame();
        m.register_slot(Slot(0), aabr(0.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.on_drag_start(Slot(0), None);
        m.on_release(cursor(500.0, 500.0));
        assert_eq!(m.take_events(), vec![Event::Dropped(Slot(0))]);
    }

    #[test]
    fn drag_release_on_source_is_drop() {
        let mut m = manager();
        m.begin_frame();
        m.register_slot(Slot(0), aabr(0.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.on_drag_start(Slot(0), None);
        m.on_release(cursor(10.0, 10.0));
        // Same-slot release still drops onto the window semantics of Conrod.
        assert_eq!(m.take_events(), vec![Event::Dropped(Slot(0))]);
    }

    #[test]
    fn split_drag_and_drop() {
        let mut m = manager();
        m.begin_frame();
        m.register_slot(Slot(0), aabr(0.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.register_slot(Slot(1), aabr(50.0, 0.0, 40.0, 40.0), cursor(50.0, 50.0));
        m.on_drag_start(Slot(0), Some(5));
        m.on_right_click(Slot(1), cursor(60.0, 20.0));
        assert_eq!(m.take_events(), vec![Event::SplitDragged(Slot(0), Slot(1))]);
        assert!(m.dragging().is_some(), "still dragging after split");

        m.on_release(cursor(500.0, 500.0));
        assert_eq!(m.take_events(), vec![Event::Dropped(Slot(0))]);
    }

    #[test]
    fn split_drop_on_open_space() {
        let mut m = manager();
        m.on_drag_start(Slot(0), Some(5));
        m.on_right_click(Slot(0), cursor(500.0, 500.0));
        assert_eq!(m.take_events(), vec![Event::SplitDropped(Slot(0))]);
    }

    #[test]
    fn empty_slot_cancels_selection() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        m.on_slot_changed(Slot(0), false);
        assert_eq!(m.selected(), None);
        m.on_click(Slot(1), true, 1, no_mods());
        m.on_drag_start(Slot(1), None);
        m.on_slot_changed(Slot(1), false);
        assert_eq!(m.dragging(), None);
    }

    #[test]
    fn mouse_over_slot_tracking() {
        let mut m = manager();
        m.begin_frame();
        m.register_slot(Slot(0), aabr(0.0, 0.0, 40.0, 40.0), cursor(20.0, 20.0));
        assert_eq!(m.mouse_over_slot(), Some(Slot(0)));
        m.register_slot(Slot(1), aabr(50.0, 0.0, 40.0, 40.0), cursor(20.0, 20.0));
        assert_eq!(m.mouse_over_slot(), Some(Slot(0)));
        m.register_slot(Slot(1), aabr(50.0, 0.0, 40.0, 40.0), cursor(60.0, 20.0));
        assert_eq!(m.mouse_over_slot(), Some(Slot(1)));
    }

    #[test]
    fn interaction_reporting() {
        let mut m = manager();
        assert_eq!(m.interaction(Slot(0)), Interaction::None);
        m.on_click(Slot(0), true, 1, no_mods());
        assert_eq!(m.interaction(Slot(0)), Interaction::Selected);
        m.on_drag_start(Slot(0), None);
        assert_eq!(m.interaction(Slot(0)), Interaction::Dragging);
    }

    #[test]
    fn use_and_drop_selected() {
        let mut m = manager();
        m.on_click(Slot(0), true, 1, no_mods());
        m.use_selected();
        assert_eq!(m.take_events(), vec![Event::Used(Slot(0))]);
        m.on_click(Slot(0), true, 1, no_mods());
        m.dropped_selected();
        assert_eq!(m.take_events(), vec![Event::Dropped(Slot(0))]);
    }
}
