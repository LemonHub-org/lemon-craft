//! Item slot widget for the Iced UI.
//!
//! Port of the Conrod `slot` widget: frame states (empty / hovered / filled /
//! selected), content image with pulse animation, amount text with SI
//! prefixes, hover highlight and drag & drop integration.
use super::{drag, image};
use crate::ui::ice::{FontId, IcedRenderer, renderer::Primitive};
use iced::{
    Color, Element, Event, Hasher, HorizontalAlignment, Layout, Length, Point, Rectangle, Size,
    VerticalAlignment, Widget, event, keyboard, layout, mouse,
};
use std::hash::Hash;
use vek::*;

const AMOUNT_SHADOW_OFFSET: [f32; 2] = [1.0, 1.0];

/// Describes how to resolve the content of a slot from game state.
pub trait SlotKey<C, I>: Copy {
    type ImageKey: PartialEq + Clone + Send + 'static;
    /// Returns `None` for an empty slot, otherwise the content key and an
    /// optional tint color.
    fn image_key(&self, source: &C) -> Option<(Self::ImageKey, Option<Color>)>;
    fn amount(&self, source: &C) -> Option<u32>;
    fn image_ids(key: &Self::ImageKey, source: &I) -> Vec<image::Handle>;
}

pub use drag::SumSlot;

/// Constraints for the size of the content image inside a slot.
#[derive(Debug, Clone, Copy)]
pub struct ContentSize {
    // Width divided by height.
    pub width_height_ratio: f32,
    // Max fraction of the slot size that each side can be.
    pub max_fraction: f32,
}

/// Computes the content size inside a slot of the given size (port of the
/// Conrod logic).
pub fn slot_content_size(slot_wh: Vec2<f32>, content: &ContentSize) -> Vec2<f32> {
    let w_max = content.max_fraction * slot_wh.x;
    let h_max = content.max_fraction * slot_wh.y;
    let max_ratio = w_max / h_max;
    if max_ratio > content.width_height_ratio {
        Vec2::new(content.width_height_ratio * h_max, h_max)
    } else {
        Vec2::new(w_max, w_max / content.width_height_ratio)
    }
}

/// Formats an amount with SI prefixes (port of the Conrod logic).
pub fn format_amount(amount: u32, use_prefixes: bool, prefix_switch_point: u32) -> String {
    if use_prefixes {
        let threshold = amount / (u32::pow(10, prefix_switch_point.saturating_sub(4)));
        match amount {
            _ if threshold >= 1_000_000_000 => format!("{}G", amount / 1_000_000_000),
            _ if threshold >= 1_000_000 => format!("{}M", amount / 1_000_000),
            _ if threshold >= 1_000 => format!("{}K", amount / 1_000),
            _ => format!("{}", amount),
        }
    } else {
        format!("{}", amount)
    }
}

/// Selects the frame index for a pulse animation (port of `animate_by_pulse`).
fn pulse_frame_index(len: usize, pulse: f32) -> usize {
    let animation_frame = (pulse * 3.0) as usize;
    animation_frame % len
}

/// Selects the image for a pulse animation (port of `animate_by_pulse`).
pub fn animate_by_pulse(images: &[image::Handle], pulse: f32) -> image::Handle {
    images[pulse_frame_index(images.len(), pulse)]
}

/// State of a slot widget; one per slot, held by the caller.
///
/// `cached_images` is behind a `RefCell` because widget `draw` runs with
/// `&self` while the content resolution may change every frame.
pub struct State<K> {
    cached_images: std::cell::RefCell<Option<(K, Vec<image::Handle>)>>,
    hover: bool,
    pressed: bool,
    press_pos: Point,
    click_count: u32,
    modifiers: keyboard::Modifiers,
}

impl<K> Default for State<K> {
    fn default() -> Self {
        Self {
            cached_images: std::cell::RefCell::new(None),
            hover: false,
            pressed: false,
            press_pos: Point::ORIGIN,
            click_count: 0,
            modifiers: keyboard::Modifiers::default(),
        }
    }
}

impl<Key> State<Key> {
    /// Re-resolves the cached images if the slot key changed.
    fn update_images<C, I, S>(&self, slot_key: S, source: &C, image_source: &I)
    where
        S: SlotKey<C, I> + Copy,
        Key: PartialEq,
        S::ImageKey: Into<Key>,
    {
        let image_key = slot_key.image_key(source).map(|(key, _)| key);
        let mut cached = self.cached_images.borrow_mut();
        let cached_changed = match (cached.as_ref().map(|(key, _)| key), &image_key) {
            (Some(cached_key), Some(new_key)) => *cached_key != new_key.clone().into(),
            (None, None) => false,
            _ => true,
        };
        if cached_changed {
            *cached = image_key.map(|key| {
                let image_ids = S::image_ids(&key, image_source);
                (key.into(), image_ids)
            });
        }
    }

    /// Whether the slot currently has content.
    pub fn filled(&self) -> bool { self.cached_images.borrow().is_some() }
}

/// A widget for displaying a single inventory slot.
pub struct Slot<'a, K, C, I, S>
where
    K: SlotKey<C, I>,
{
    slot_key: K,

    // Images for slot background and frame.
    empty_slot: image::Handle,
    hovered_slot: image::Handle,
    selected_slot: image::Handle,
    filled_slot: image::Handle,
    background_color: Option<Rgba<u8>>,

    // Size of the content image.
    content_size: Vec2<f32>,
    selected_content_scale: f32,

    icon: Option<(image::Handle, Vec2<f32>, Option<Rgba<u8>>)>,

    // Amount styling.
    amount_font: FontId,
    amount_font_size: u16,
    amount_margins: Vec2<f32>,
    amount_text_color: Color,

    use_prefixes: bool,
    prefix_switch_point: u32,

    // Menu button navigation.
    menu_hover: bool,

    content_source: &'a C,
    image_source: &'a I,

    pulse: f32,

    drag: Option<&'a std::cell::RefCell<drag::DragManager<S>>>,

    width: Length,
    height: Length,

    state: &'a mut State<K::ImageKey>,
}

impl<'a, K, C, I, S> Slot<'a, K, C, I, S>
where
    K: SlotKey<C, I> + Into<S>,
    S: SumSlot,
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        slot_key: K,
        empty_slot: image::Handle,
        hovered_slot: image::Handle,
        selected_slot: image::Handle,
        filled_slot: image::Handle,
        content_size: Vec2<f32>,
        selected_content_scale: f32,
        amount_font: FontId,
        amount_font_size: u16,
        amount_margins: Vec2<f32>,
        amount_text_color: Color,
        content_source: &'a C,
        image_source: &'a I,
        pulse: f32,
        state: &'a mut State<K::ImageKey>,
    ) -> Self {
        Self {
            slot_key,
            empty_slot,
            hovered_slot,
            selected_slot,
            filled_slot,
            background_color: None,
            content_size,
            selected_content_scale,
            icon: None,
            amount_font,
            amount_font_size,
            amount_margins,
            amount_text_color,
            use_prefixes: true,
            prefix_switch_point: 6,
            menu_hover: false,
            content_source,
            image_source,
            pulse,
            drag: None,
            width: Length::Shrink,
            height: Length::Shrink,
            state,
        }
    }

    #[must_use]
    pub fn with_background_color(mut self, color: Rgba<u8>) -> Self {
        self.background_color = Some(color);
        self
    }

    #[must_use]
    pub fn with_manager(mut self, drag: &'a std::cell::RefCell<drag::DragManager<S>>) -> Self {
        self.drag = Some(drag);
        self
    }

    #[must_use]
    pub fn filled_slot(mut self, img: image::Handle) -> Self {
        self.filled_slot = img;
        self
    }

    #[must_use]
    pub fn with_icon(
        mut self,
        img: image::Handle,
        size: Vec2<f32>,
        color: Option<Rgba<u8>>,
    ) -> Self {
        self.icon = Some((img, size, color));
        self
    }

    #[must_use]
    pub fn use_prefixes(mut self, use_prefixes: bool) -> Self {
        self.use_prefixes = use_prefixes;
        self
    }

    #[must_use]
    pub fn prefix_switch_point(mut self, prefix_switch_point: u32) -> Self {
        self.prefix_switch_point = prefix_switch_point;
        self
    }

    #[must_use]
    pub fn menu_hover(mut self, menu_hover: bool) -> Self {
        self.menu_hover = menu_hover;
        self
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

/// Factory for constructing many slots with shared settings.
pub struct SlotMaker<'a, C, I, S> {
    pub empty_slot: image::Handle,
    pub hovered_slot: image::Handle,
    pub filled_slot: image::Handle,
    pub selected_slot: image::Handle,
    pub background_color: Option<Rgba<u8>>,
    pub content_size: ContentSize,
    pub selected_content_scale: f32,
    pub amount_font: FontId,
    pub amount_font_size: u16,
    pub amount_margins: Vec2<f32>,
    pub amount_text_color: Color,
    pub content_source: &'a C,
    pub image_source: &'a I,
    pub drag: Option<&'a std::cell::RefCell<drag::DragManager<S>>>,
    pub pulse: f32,
}

impl<'a, C, I, S> SlotMaker<'a, C, I, S>
where
    S: SumSlot,
{
    /// Creates a new [`Slot`] widget for the given contents.
    pub fn fabricate<K: SlotKey<C, I> + Into<S>>(
        &self,
        contents: K,
        wh: Vec2<f32>,
        menu_hover: bool,
        state: &'a mut State<K::ImageKey>,
    ) -> Slot<'a, K, C, I, S> {
        Slot::new(
            contents,
            self.empty_slot,
            self.hovered_slot,
            self.selected_slot,
            self.filled_slot,
            slot_content_size(wh, &self.content_size),
            self.selected_content_scale,
            self.amount_font,
            self.amount_font_size,
            self.amount_margins,
            self.amount_text_color,
            self.content_source,
            self.image_source,
            self.pulse,
            state,
        )
        .width(Length::Units(wh.x as u16))
        .height(Length::Units(wh.y as u16))
        .menu_hover(menu_hover)
        .with_background_color_opt(self.background_color)
        .with_manager_opt(self.drag)
    }
}

impl<'a, K, C, I, S> Slot<'a, K, C, I, S>
where
    K: SlotKey<C, I>,
{
    fn with_background_color_opt(mut self, color: Option<Rgba<u8>>) -> Self {
        self.background_color = color;
        self
    }

    fn with_manager_opt(
        mut self,
        drag: Option<&'a std::cell::RefCell<drag::DragManager<S>>>,
    ) -> Self {
        self.drag = drag;
        self
    }
}

impl<M, K, C, I, S> Widget<M, IcedRenderer> for Slot<'_, K, C, I, S>
where
    K: SlotKey<C, I> + Into<S>,
    S: SumSlot,
{
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, _renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();

        // Refresh the cached content images (key change or first frame).
        self.state
            .update_images(self.slot_key, self.content_source, self.image_source);

        if let Some(drag) = self.drag {
            let mut drag = drag.borrow_mut();
            // Cancel selection/dragging if this slot became empty.
            drag.on_slot_changed(self.slot_key.into(), self.state.filled());
            drag.register_slot(
                self.slot_key.into(),
                aabr_from_bounds(bounds),
                cursor_position,
            );
        }

        let interaction = self.drag.as_ref().map_or(drag::Interaction::None, |d| {
            d.borrow().interaction(self.slot_key.into())
        });

        // No content shown while the slot is being dragged.
        let content_images = if interaction == drag::Interaction::Dragging {
            None
        } else {
            self.state
                .cached_images
                .borrow()
                .as_ref()
                .map(|(_, imgs)| imgs.clone())
        };

        let slot_image = match interaction {
            drag::Interaction::Selected => self.selected_slot,
            _ if content_images.is_some() => self.filled_slot,
            _ => self.empty_slot,
        };

        let mut primitives = vec![renderer.draw_image(
            slot_image,
            bounds,
            self.background_color.unwrap_or(Rgba::broadcast(255)),
        )];

        // Icon (only when there is no content).
        if let (Some((icon_image, size, color)), false) = (self.icon, content_images.is_some()) {
            let icon_bounds = centered_rect(bounds, size);
            primitives.push(renderer.draw_image(
                icon_image,
                icon_bounds,
                color.unwrap_or(Rgba::broadcast(255)),
            ));
        }

        // Contents.
        if let Some(content_images) = content_images.as_ref() {
            let scale = if interaction == drag::Interaction::Selected {
                self.selected_content_scale
            } else {
                1.0
            };
            let content_bounds = centered_rect(bounds, self.content_size * scale);
            primitives.push(renderer.draw_image(
                animate_by_pulse(content_images, self.pulse),
                content_bounds,
                Rgba::broadcast(255),
            ));
        }

        // Hover highlight.
        if self.state.hover || self.menu_hover {
            primitives.push(renderer.draw_image(self.hovered_slot, bounds, Rgba::broadcast(255)));
        }

        // Ghost image while dragging.
        if interaction == drag::Interaction::Dragging
            && let (Some(drag), Some(content_images)) =
                (self.drag, self.state.cached_images.borrow().as_ref())
        {
            let drag = drag.borrow();
            let size = match self.slot_key.into().drag_size() {
                Some(size) => Vec2::from(size),
                None => drag.drag_img_size(),
            };
            let ghost_bounds = Rectangle {
                x: cursor_position.x - size.x / 2.0,
                y: cursor_position.y - size.y / 2.0,
                width: size.x,
                height: size.y,
            };
            primitives.push(renderer.draw_image(
                animate_by_pulse(content_images.1.as_ref(), self.pulse),
                ghost_bounds,
                Rgba::broadcast(255),
            ));
        }

        // Amount text.
        if interaction != drag::Interaction::Dragging
            && let Some(amount) = self.slot_key.amount(self.content_source)
        {
            let amount = format_amount(amount, self.use_prefixes, self.prefix_switch_point);
            let content_bounds = centered_rect(bounds, self.content_size);
            primitives.extend(draw_amount_text(
                renderer,
                defaults,
                &amount,
                content_bounds,
                self.amount_font,
                self.amount_font_size,
                self.amount_margins,
                self.amount_text_color,
            ));
        }

        let mouse_interaction = if self.state.hover && self.drag.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        };

        (Primitive::Group { primitives }, mouse_interaction)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        std::hash::Hash::hash(&self.content_size.map(|v| v.to_bits()), state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn iced::Clipboard,
        _messages: &mut Vec<M>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let in_bounds = bounds.contains(cursor_position);

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if in_bounds != self.state.hover {
                    self.state.hover = in_bounds;
                }
                if let Some(drag) = self.drag {
                    let mut drag = drag.borrow_mut();
                    if self.state.pressed && drag.dragging().is_none() {
                        let moved = (cursor_position.x - self.state.press_pos.x).abs()
                            + (cursor_position.y - self.state.press_pos.y).abs();
                        if moved > DRAG_THRESHOLD && self.state.filled() {
                            drag.on_drag_start(
                                self.slot_key.into(),
                                self.slot_key.amount(self.content_source),
                            );
                        }
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if in_bounds && !self.state.pressed {
                    self.state.pressed = true;
                    self.state.press_pos = cursor_position;
                    self.state.click_count += 1;
                    if let Some(drag) = self.drag {
                        let mut drag = drag.borrow_mut();
                        drag.on_click(
                            self.slot_key.into(),
                            self.state.filled(),
                            self.state.click_count,
                            self.state.modifiers,
                        );
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.state.pressed = false;
                if let Some(drag) = self.drag {
                    drag.borrow_mut().on_release(cursor_position);
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if in_bounds && let Some(drag) = self.drag {
                    drag.borrow_mut()
                        .on_right_click(self.slot_key.into(), cursor_position);
                }
            },
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.state.modifiers = modifiers;
            },
            _ => {},
        }

        event::Status::Ignored
    }
}

const DRAG_THRESHOLD: f32 = 4.0;

fn centered_rect(bounds: Rectangle, size: Vec2<f32>) -> Rectangle {
    Rectangle {
        x: bounds.x + (bounds.width - size.x) / 2.0,
        y: bounds.y + (bounds.height - size.y) / 2.0,
        width: size.x,
        height: size.y,
    }
}

fn aabr_from_bounds(bounds: Rectangle) -> Aabr<i32> {
    let min = Vec2::new(bounds.x.trunc() as i32, bounds.y.trunc() as i32);
    let max = min + Vec2::new(bounds.width.trunc() as i32, bounds.height.trunc() as i32);
    Aabr { min, max }
}

fn draw_amount_text(
    renderer: &mut IcedRenderer,
    defaults: &<IcedRenderer as iced::Renderer>::Defaults,
    amount: &str,
    content_bounds: Rectangle,
    font: FontId,
    font_size: u16,
    margins: Vec2<f32>,
    color: Color,
) -> Vec<Primitive> {
    use iced::text::Renderer as _;

    let (tw, th) = renderer.measure(
        amount,
        font_size,
        font,
        Size::new(content_bounds.width, content_bounds.height),
    );

    let x = content_bounds.x + content_bounds.width - margins.x - tw;
    let y = content_bounds.y + content_bounds.height - margins.y - th;

    let shadow_bounds = Rectangle {
        x: x + AMOUNT_SHADOW_OFFSET[0],
        y: y + AMOUNT_SHADOW_OFFSET[1],
        width: tw,
        height: th,
    };
    let text_bounds = Rectangle {
        x,
        y,
        width: tw,
        height: th,
    };

    let (shadow, _) = <IcedRenderer as iced::text::Renderer>::draw(
        renderer,
        defaults,
        shadow_bounds,
        amount,
        font_size,
        font,
        Some(Color::BLACK),
        HorizontalAlignment::Left,
        VerticalAlignment::Top,
    );
    let (text, _) = <IcedRenderer as iced::text::Renderer>::draw(
        renderer,
        defaults,
        text_bounds,
        amount,
        font_size,
        font,
        Some(color),
        HorizontalAlignment::Left,
        VerticalAlignment::Top,
    );

    vec![shadow, text]
}

impl<'a, M, K, C, I, S> From<Slot<'a, K, C, I, S>> for Element<'a, M, IcedRenderer>
where
    K: SlotKey<C, I> + Into<S> + 'a,
    S: SumSlot + 'a,
    C: 'a,
    I: 'a,
    M: 'a,
{
    fn from(slot: Slot<'a, K, C, I, S>) -> Element<'a, M, IcedRenderer> { Element::new(slot) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_size_keeps_aspect_ratio() {
        let content = ContentSize {
            width_height_ratio: 1.0,
            max_fraction: 0.8,
        };
        let size = slot_content_size(Vec2::new(50.0, 40.0), &content);
        assert!((size.x / size.y - 1.0).abs() < 0.001, "square content");
        assert!(size.x <= 40.0 && size.y <= 32.0, "bounded by max_fraction");
    }

    #[test]
    fn content_size_wide_slot() {
        let content = ContentSize {
            width_height_ratio: 2.0,
            max_fraction: 0.8,
        };
        let size = slot_content_size(Vec2::new(100.0, 40.0), &content);
        assert!((size.x / size.y - 2.0).abs() < 0.001);
        assert!(size.y <= 32.0);
    }

    #[test]
    fn amount_plain_format() {
        assert_eq!(format_amount(42, false, 6), "42");
        assert_eq!(format_amount(0, false, 6), "0");
    }

    #[test]
    fn amount_prefix_format() {
        // With the default switch point of 6, values below 100_000
        // (amount / 10^(6-4) < 1000) stay un-prefixed (Conrod semantics).
        assert_eq!(format_amount(999, true, 6), "999");
        assert_eq!(format_amount(1500, true, 6), "1500");
        assert_eq!(format_amount(1_500_000, true, 6), "1500K");
        assert_eq!(format_amount(2_000_000_000, true, 6), "2000M");
    }

    #[test]
    fn amount_prefix_switch_point() {
        // With a switch point of 4, prefixes kick in earlier.
        assert_eq!(format_amount(999, true, 4), "999");
        assert_eq!(format_amount(1500, true, 4), "1K");
        assert_eq!(format_amount(1_500_000, true, 4), "1M");
    }

    #[test]
    fn pulse_selects_frame() {
        assert_eq!(pulse_frame_index(3, 0.0), 0);
        assert_eq!(pulse_frame_index(3, 0.5), 1);
        assert_eq!(pulse_frame_index(3, 1.0), 3 % 3);
    }
}
