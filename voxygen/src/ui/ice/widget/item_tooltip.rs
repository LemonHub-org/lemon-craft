//! Item tooltip support for the Iced UI.
//!
//! Port of the Conrod `item_tooltip` machinery: a hover/fade state machine,
//! mouse-relative positioning, and a content widget that renders the tooltip
//! frame, item image and text rows.
use super::{image, image_frame::nine_slice_bounds};
use crate::ui::ice::{FontId, IcedRenderer, renderer::Primitive};
use iced::{
    Color, Element, Event, Hasher, HorizontalAlignment, Layout, Length, Point, Rectangle, Size,
    VerticalAlignment, Widget, event, layout, mouse, overlay,
};
use std::{
    hash::Hash,
    sync::Mutex,
    time::{Duration, Instant},
};
use vek::*;

// Spacing between the tooltip and the mouse cursor.
const MOUSE_PAD_Y: f32 = 15.0;
// Vertical spacing between tooltip content rows.
const V_PAD: f32 = 10.0;
// Horizontal padding of the tooltip content.
const H_PAD: f32 = 10.0;
// Default tooltip width.
pub const DEFAULT_WIDTH: f32 = 320.0;

/// A row of tooltip text with an optional color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatLine<'a> {
    pub text: &'a str,
    pub color: Option<Color>,
}

impl<'a> From<&'a str> for StatLine<'a> {
    fn from(text: &'a str) -> Self { Self { text, color: None } }
}

/// Positions a tooltip of the given size next to the mouse cursor.
///
/// Port of the Conrod logic (coordinates are top-left origin): the tooltip
/// flows to the left of the cursor when there is room, and below it otherwise.
pub fn tooltip_position(
    tooltip_size: Vec2<f32>,
    mouse: Vec2<f32>,
    window: Vec2<f32>,
    pad_y: f32,
) -> Vec2<f32> {
    let [t_w, t_h] = tooltip_size.into_array();
    let [m_x, m_y] = mouse.into_array();
    let [_, w_h] = window.into_array();

    let x = if m_x > t_w / 2.0 {
        m_x - t_w / 2.0
    } else {
        m_x + t_w / 2.0
    };
    let y = if w_h - m_y > t_h + pad_y {
        m_y + pad_y + t_h / 2.0
    } else {
        m_y - pad_y - t_h / 2.0
    };
    Vec2::new(x, y)
}

#[derive(Clone, Copy)]
struct Hover<K>(K, Vec2<f32>);

#[derive(Clone, Copy)]
enum HoverState<K> {
    // `allow`ed: rustc's dead-code analysis misreports this variant as never
    // constructed when it is only reconstructed inside match arms on the same
    // enum (see `maintain`), so the lint is a false positive here.
    #[allow(dead_code)]
    Fading(Instant, Hover<K>, Option<(Instant, K)>),
    Start(Instant, K),
    None,
}

/// Hover/fade state machine for item tooltips (port of the Conrod
/// `ItemTooltipManager`).
///
/// Widgets report hover through [`ItemTooltipManager::update`]; call
/// [`ItemTooltipManager::maintain`] once per frame before building the UI.
pub struct ItemTooltipManager<K> {
    state: HoverState<K>,
    // `Some(Some(..))` = the mouse is over a widget, `Some(None)` = it left,
    // `None` = no widget reported this frame.
    update: Mutex<Option<Option<(K, Vec2<f32>)>>>,
    hover_pos: Vec2<f32>,
    // How long before a tooltip is displayed when hovering.
    hover_dur: Duration,
    // How long it takes a tooltip to disappear.
    fade_dur: Duration,
}

impl<K> ItemTooltipManager<K> {
    pub fn new(hover_dur: Duration, fade_dur: Duration) -> Self {
        Self {
            state: HoverState::None,
            update: Mutex::new(None),
            hover_pos: Vec2::zero(),
            hover_dur,
            fade_dur,
        }
    }

    /// Reports that the mouse is hovering the tooltipped widget `key` (or
    /// pass `None` when the mouse left it). Called from widget `draw`.
    pub fn update(&self, hover: Option<(K, Vec2<f32>)>) {
        *self.update.lock().unwrap() = Some(hover);
    }
}

impl<K: PartialEq + Copy> ItemTooltipManager<K> {
    /// Advances the state machine; call once per frame.
    pub fn maintain(&mut self) {
        let hover = self.update.get_mut().unwrap().take();
        let state = self.state;
        self.state = match hover {
            // The mouse is over a tooltipped widget.
            Some(Some((key, pos))) => {
                self.hover_pos = pos;
                match state {
                    HoverState::Fading(_, _, Some((_, id))) if id == key => state,
                    HoverState::Fading(start, hover, _) => {
                        HoverState::Fading(start, hover, Some((Instant::now(), key)))
                    },
                    HoverState::Start(_, id) if id == key => state,
                    // A different widget is hovered: fade out the old tooltip.
                    HoverState::Start(_, id) => HoverState::Fading(
                        Instant::now(),
                        Hover(id, pos),
                        Some((Instant::now(), key)),
                    ),
                    HoverState::None => HoverState::Start(Instant::now(), key),
                }
            },
            // The mouse left the widget: fade out the displayed tooltip.
            Some(None) => match state {
                HoverState::Start(_, id) => {
                    HoverState::Fading(Instant::now(), Hover(id, self.hover_pos), None)
                },
                HoverState::Fading(start, hover, _) => HoverState::Fading(start, hover, None),
                HoverState::None => state,
            },
            // No widget reported this frame: keep the current state.
            None => state,
        };

        // Handle fade timing.
        if let HoverState::Fading(start, _, maybe_hover) = self.state
            && start.elapsed() > self.fade_dur
        {
            self.state = match maybe_hover {
                Some((start, hover)) => HoverState::Start(start, hover),
                None => HoverState::None,
            };
        }
    }

    /// Returns the hover position and transparency for the tooltip of `key`,
    /// if it should currently be shown.
    pub fn showing(&self, key: K) -> Option<(Vec2<f32>, f32)> {
        match self.state {
            HoverState::Fading(start, Hover(id, pos), _) if id == key => {
                let transparency =
                    (1.0 - start.elapsed().as_secs_f32() / self.fade_dur.as_secs_f32()).max(0.0);
                (transparency > 0.0).then_some((pos, transparency))
            },
            HoverState::Start(start, id) if id == key && start.elapsed() > self.hover_dur => {
                Some((self.hover_pos, 1.0))
            },
            _ => None,
        }
    }
}

/// A wrapper that shows an item tooltip while the wrapped content is hovered.
pub struct WithItemTooltip<'a, M> {
    content: Element<'a, M, IcedRenderer>,
    tooltip: Element<'a, M, IcedRenderer>,
    manager: &'a ItemTooltipManager<Aabr<i32>>,
    window_size: Vec2<f32>,
}

impl<'a, M> WithItemTooltip<'a, M> {
    pub fn new<C, T>(
        content: C,
        tooltip: T,
        manager: &'a ItemTooltipManager<Aabr<i32>>,
        window_size: Vec2<f32>,
    ) -> Self
    where
        C: Into<Element<'a, M, IcedRenderer>>,
        T: Into<Element<'a, M, IcedRenderer>>,
    {
        Self {
            content: content.into(),
            tooltip: tooltip.into(),
            manager,
            window_size,
        }
    }
}

impl<M> Widget<M, IcedRenderer> for WithItemTooltip<'_, M> {
    fn width(&self) -> Length { self.content.width() }

    fn height(&self) -> Length { self.content.height() }

    fn layout(&self, renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        self.content.layout(renderer, limits)
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();
        let hover = bounds.contains(cursor_position).then(|| {
            (
                aabr_from_bounds(bounds),
                Vec2::new(cursor_position.x, cursor_position.y),
            )
        });
        self.manager.update(hover);
        self.content
            .draw(renderer, defaults, layout, cursor_position, viewport)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.content.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &IcedRenderer,
        clipboard: &mut dyn iced::Clipboard,
        messages: &mut Vec<M>,
    ) -> event::Status {
        self.content.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        )
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, M, IcedRenderer>> {
        let key = aabr_from_bounds(layout.bounds());
        self.manager.showing(key).map(|(mouse_pos, transparency)| {
            overlay::Element::new(
                Point::ORIGIN,
                Box::new(TooltipOverlay::new(
                    &self.tooltip,
                    mouse_pos,
                    transparency,
                    self.window_size,
                )),
            )
        })
    }
}

struct TooltipOverlay<'a, M> {
    content: &'a Element<'a, M, IcedRenderer>,
    mouse_pos: Vec2<f32>,
    transparency: f32,
    window_size: Vec2<f32>,
}

impl<'a, M> TooltipOverlay<'a, M> {
    fn new(
        content: &'a Element<'a, M, IcedRenderer>,
        mouse_pos: Vec2<f32>,
        transparency: f32,
        window_size: Vec2<f32>,
    ) -> Self {
        Self {
            content,
            mouse_pos,
            transparency,
            window_size,
        }
    }
}

impl<M> overlay::Overlay<M, IcedRenderer> for TooltipOverlay<'_, M> {
    fn layout(&self, renderer: &IcedRenderer, bounds: Size, position: Point) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let mut node = self.content.layout(renderer, &limits);
        let size = node.size();

        // Position relative to the overlay origin.
        let rel_mouse = Vec2::new(self.mouse_pos.x + position.x, self.mouse_pos.y + position.y);
        let pos = tooltip_position(
            Vec2::new(size.width, size.height),
            rel_mouse,
            self.window_size,
            MOUSE_PAD_Y,
        );
        node.move_to(Point { x: pos.x, y: pos.y });
        node
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let (primitive, interaction) =
            self.content
                .draw(renderer, defaults, layout, cursor_position, &Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: self.window_size.x,
                    height: self.window_size.y,
                });
        (
            Primitive::Opacity {
                alpha: self.transparency,
                content: Box::new(primitive),
            },
            interaction,
        )
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        (position.x as u32).hash(state);
        (position.y as u32).hash(state);
        (self.mouse_pos.x as i32).hash(state);
        (self.mouse_pos.y as i32).hash(state);
        self.content.hash_layout(state);
    }
}

fn aabr_from_bounds(bounds: Rectangle) -> Aabr<i32> {
    let min = Vec2::new(bounds.x.trunc() as i32, bounds.y.trunc() as i32);
    let max = min + Vec2::new(bounds.width.trunc() as i32, bounds.height.trunc() as i32);
    Aabr { min, max }
}

/// Content of an item tooltip: frame, optional image and text rows.
pub struct ItemTooltip<'a> {
    // Frame images [top, bottom, right, left] and corners [tr, tl, br, bl].
    frame_edges: [image::Handle; 4],
    frame_corners: [image::Handle; 4],
    frame_color: Option<Rgba<u8>>,
    border_size: f32,
    center: Color,

    title: &'a str,
    title_color: Color,
    subtitle: Option<&'a str>,
    quantity: Option<&'a str>,
    image: Option<(image::Handle, Vec2<f32>)>,
    stats: Vec<StatLine<'a>>,
    desc: Option<&'a str>,

    font: FontId,
    font_size: u16,
    width: Length,
}

impl<'a> ItemTooltip<'a> {
    /// Creates a new tooltip with the given 9-slice frame images.
    pub fn new(
        frame_edges: [image::Handle; 4],
        frame_corners: [image::Handle; 4],
        border_size: f32,
    ) -> Self {
        Self {
            frame_edges,
            frame_corners,
            frame_color: None,
            border_size,
            center: Color::from_rgb8(0x21, 0x21, 0x17),
            title: "",
            title_color: Color::WHITE,
            subtitle: None,
            quantity: None,
            image: None,
            stats: Vec::new(),
            desc: None,
            font: FontId::default(),
            font_size: 14,
            width: Length::Units(DEFAULT_WIDTH as u16),
        }
    }

    #[must_use]
    pub fn frame_color(mut self, color: Rgba<u8>) -> Self {
        self.frame_color = Some(color);
        self
    }

    #[must_use]
    pub fn center(mut self, color: Color) -> Self {
        self.center = color;
        self
    }

    #[must_use]
    pub fn title(mut self, title: &'a str, color: Color) -> Self {
        self.title = title;
        self.title_color = color;
        self
    }

    #[must_use]
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    #[must_use]
    pub fn quantity(mut self, quantity: &'a str) -> Self {
        self.quantity = Some(quantity);
        self
    }

    #[must_use]
    pub fn image(mut self, handle: image::Handle, dims: Vec2<f32>) -> Self {
        self.image = Some((handle, dims));
        self
    }

    #[must_use]
    pub fn stats(mut self, stats: Vec<StatLine<'a>>) -> Self {
        self.stats = stats;
        self
    }

    #[must_use]
    pub fn desc(mut self, desc: &'a str) -> Self {
        self.desc = Some(desc);
        self
    }

    #[must_use]
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    #[must_use]
    pub fn font_size(mut self, font_size: u16) -> Self {
        self.font_size = font_size;
        self
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    fn inner_height(&self, renderer: &IcedRenderer) -> f32 {
        use iced::text::Renderer as _;

        let bounds = Size::new(DEFAULT_WIDTH - H_PAD * 2.0, f32::MAX);
        let mut height = V_PAD;
        if let Some((_, dims)) = self.image {
            height += dims.y;
        }
        let (_, title_h) = renderer.measure(self.title, self.font_size, self.font, bounds);
        height += title_h;
        if self.quantity.is_some() {
            height += self.line_height(renderer) + 2.0;
        }
        if self.subtitle.is_some() {
            height += self.line_height(renderer) + 2.0;
        }
        for _ in &self.stats {
            height += self.line_height(renderer) + 4.0;
        }
        if let Some(desc) = self.desc {
            let (_, h) = renderer.measure(desc, self.font_size, self.font, bounds);
            height += h + V_PAD;
        }
        height + V_PAD
    }

    fn line_height(&self, renderer: &IcedRenderer) -> f32 {
        use iced::text::Renderer as _;
        let (_, h) = renderer.measure(
            "x",
            self.font_size,
            self.font,
            Size::new(DEFAULT_WIDTH - H_PAD * 2.0, f32::MAX),
        );
        h
    }
}

impl<M> Widget<M, IcedRenderer> for ItemTooltip<'_> {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { Length::Shrink }

    fn layout(&self, renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width);
        let height = self.inner_height(renderer);
        layout::Node::new(limits.resolve(Size::new(0.0, height)))
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        use iced::text::Renderer as _;

        let bounds = layout.bounds();
        let mut primitives = Vec::new();

        // 9-slice frame.
        let slices = nine_slice_bounds(bounds, self.border_size);
        let frame_color = self.frame_color.unwrap_or(Rgba::broadcast(255));
        for (i, handle) in self.frame_edges.iter().enumerate() {
            if let Some(rect) = slices[edge_index(i)] {
                primitives.push(renderer.draw_image(*handle, rect, frame_color));
            }
        }
        for (i, handle) in self.frame_corners.iter().enumerate() {
            if let Some(rect) = slices[corner_index(i)] {
                primitives.push(renderer.draw_image(*handle, rect, frame_color));
            }
        }
        if let Some(rect) = slices[8] {
            primitives.push(renderer.draw_rectangle(
                rect,
                Rgba::new(self.center.r, self.center.g, self.center.b, self.center.a),
            ));
        }

        let mut y_cursor = bounds.y + V_PAD;
        let x_cursor = bounds.x + H_PAD;
        let text_width = bounds.width - H_PAD * 2.0;

        // Image.
        if let Some((handle, dims)) = self.image {
            primitives.push(renderer.draw_image(
                handle,
                Rectangle {
                    x: x_cursor,
                    y: y_cursor,
                    width: dims.x,
                    height: dims.y,
                },
                Rgba::broadcast(255),
            ));
            y_cursor += dims.y;
        }

        // Title.
        let text_bounds = Size::new(text_width, f32::MAX);
        let (_, title_h) = renderer.measure(self.title, self.font_size, self.font, text_bounds);
        primitives.extend(draw_text_line(
            renderer,
            defaults,
            self.title,
            Rectangle {
                x: x_cursor,
                y: y_cursor,
                width: text_width,
                height: title_h,
            },
            self.font,
            self.font_size,
            self.title_color,
        ));
        y_cursor += title_h;

        // Quantity / subtitle.
        for line in [self.quantity, self.subtitle].into_iter().flatten() {
            let (_, h) = renderer.measure(line, self.font_size, self.font, text_bounds);
            primitives.extend(draw_text_line(
                renderer,
                defaults,
                line,
                Rectangle {
                    x: x_cursor,
                    y: y_cursor + 2.0,
                    width: text_width,
                    height: h,
                },
                self.font,
                self.font_size,
                Color::from_rgb8(0x80, 0x80, 0x80),
            ));
            y_cursor += h + 2.0;
        }

        // Stats.
        for stat in &self.stats {
            let (_, h) = renderer.measure(stat.text, self.font_size, self.font, text_bounds);
            primitives.extend(draw_text_line(
                renderer,
                defaults,
                stat.text,
                Rectangle {
                    x: x_cursor,
                    y: y_cursor + 4.0,
                    width: text_width,
                    height: h,
                },
                self.font,
                self.font_size,
                stat.color.unwrap_or(Color::WHITE),
            ));
            y_cursor += h + 4.0;
        }

        // Description.
        if let Some(desc) = self.desc {
            let (_, h) = renderer.measure(desc, self.font_size, self.font, text_bounds);
            primitives.extend(draw_text_line(
                renderer,
                defaults,
                desc,
                Rectangle {
                    x: x_cursor,
                    y: y_cursor + V_PAD,
                    width: text_width,
                    height: h,
                },
                self.font,
                self.font_size,
                Color::from_rgb8(0x80, 0x80, 0x80),
            ));
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.title.hash(state);
        self.subtitle.hash(state);
        self.quantity.hash(state);
        self.width.hash(state);
        self.font_size.hash(state);
        self.stats.len().hash(state);
    }
}

fn draw_text_line(
    renderer: &mut IcedRenderer,
    defaults: &<IcedRenderer as iced::Renderer>::Defaults,
    content: &str,
    bounds: Rectangle,
    font: FontId,
    font_size: u16,
    color: Color,
) -> Vec<Primitive> {
    let (primitive, _) = <IcedRenderer as iced::text::Renderer>::draw(
        renderer,
        defaults,
        bounds,
        content,
        font_size,
        font,
        Some(color),
        HorizontalAlignment::Left,
        VerticalAlignment::Top,
    );
    vec![primitive]
}

// Edge order is [top, bottom, right, left]; slice order is [right, tr, top,
// tl, left, bl, bottom, br, center].
fn edge_index(i: usize) -> usize { [2, 0, 4, 6][i] }

// Corner order is [tr, tl, br, bl].
fn corner_index(i: usize) -> usize { [1, 3, 7, 5][i] }

impl<'a, M> From<ItemTooltip<'a>> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(tooltip: ItemTooltip<'a>) -> Element<'a, M, IcedRenderer> { Element::new(tooltip) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_position_flows_left_when_room() {
        let pos = tooltip_position(
            Vec2::new(320.0, 100.0),
            Vec2::new(960.0, 540.0),
            Vec2::new(1920.0, 1080.0),
            15.0,
        );
        assert_eq!(pos.x, 960.0 - 160.0, "centered left of the cursor");
        assert!(pos.y > 540.0, "below the cursor");
    }

    #[test]
    fn tooltip_position_flows_right_near_left_edge() {
        let pos = tooltip_position(
            Vec2::new(320.0, 100.0),
            Vec2::new(60.0, 540.0),
            Vec2::new(1920.0, 1080.0),
            15.0,
        );
        assert_eq!(pos.x, 60.0 + 160.0, "flips to the right of the cursor");
    }

    #[test]
    fn tooltip_position_flows_above_near_bottom() {
        let pos = tooltip_position(
            Vec2::new(320.0, 100.0),
            Vec2::new(960.0, 1000.0),
            Vec2::new(1920.0, 1080.0),
            15.0,
        );
        assert!(pos.y < 1000.0, "above the cursor near the bottom edge");
    }

    #[test]
    fn manager_hover_and_fade() {
        let mut manager =
            ItemTooltipManager::<u8>::new(Duration::from_millis(50), Duration::from_millis(20));
        let pos = Vec2::new(10.0, 10.0);

        assert!(manager.showing(1).is_none());
        manager.update(Some((1, pos)));
        manager.maintain();
        assert!(manager.showing(1).is_none(), "not shown before hover_dur");
        std::thread::sleep(Duration::from_millis(60));
        manager.maintain();
        let (shown_pos, alpha) = manager.showing(1).unwrap();
        assert_eq!(shown_pos, pos);
        assert_eq!(alpha, 1.0);

        // Moving to a different key starts fading.
        manager.update(Some((2, pos)));
        manager.maintain();
        let (_, alpha) = manager.showing(1).unwrap();
        assert!(alpha < 1.0, "fading");
        std::thread::sleep(Duration::from_millis(30));
        manager.update(Some((2, pos)));
        manager.maintain();
        assert!(manager.showing(1).is_none(), "faded out");

        // Leaving the widget entirely.
        manager.update(None);
        manager.maintain();
        let (_, alpha) = manager.showing(2).unwrap();
        assert!(alpha < 1.0, "fading after cursor leaves");
        std::thread::sleep(Duration::from_millis(30));
        manager.maintain();
        assert!(manager.showing(2).is_none());
    }

    #[test]
    fn manager_restart_on_new_hover() {
        let mut manager =
            ItemTooltipManager::<u8>::new(Duration::from_millis(50), Duration::from_millis(20));
        manager.update(Some((1, Vec2::zero())));
        manager.maintain();
        std::thread::sleep(Duration::from_millis(60));
        manager.maintain();
        assert!(manager.showing(1).is_some());

        // Hovering a new widget and waiting fades out the old one.
        manager.update(Some((2, Vec2::zero())));
        manager.maintain();
        std::thread::sleep(Duration::from_millis(30));
        manager.maintain();
        std::thread::sleep(Duration::from_millis(60));
        manager.maintain();
        assert!(manager.showing(2).is_some());
        assert!(manager.showing(1).is_none());
    }
}
