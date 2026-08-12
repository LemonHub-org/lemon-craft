//! Anchors a widget to a 3D world position (port of the Conrod `Ingame`
//! widget).
//!
//! The wrapped content keeps its normal UI layout but is drawn projected at
//! `pos` through the renderer's view-projection matrix, with frustum culling.
use crate::ui::ice::IcedRenderer;
use iced::{Element, Event, Hasher, Layout, Length, Point, Rectangle, Widget, event, layout};
use std::hash::Hash;
use vek::*;

/// Positions the wrapped widget as if it was at `pos` in the 3D game world.
///
/// Note: the widget size is not scaled based on distance for performance and
/// stylistic purposes (mirrors the Conrod `Ingame` behavior).
pub struct WorldAnchor<'a, M> {
    pos: Vec3<f32>,
    content: Element<'a, M, IcedRenderer>,
}

impl<'a, M> WorldAnchor<'a, M> {
    pub fn new(pos: Vec3<f32>, content: impl Into<Element<'a, M, IcedRenderer>>) -> Self {
        Self {
            pos,
            content: content.into(),
        }
    }
}

impl<M> Widget<M, IcedRenderer> for WorldAnchor<'_, M> {
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
        let (primitive, interaction) =
            self.content
                .draw(renderer, defaults, layout, cursor_position, viewport);
        (
            Primitive::WorldPos {
                pos: self.pos,
                dims: Vec2::new(bounds.width, bounds.height),
                content: Box::new(primitive),
            },
            interaction,
        )
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

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<iced::overlay::Element<'_, M, IcedRenderer>> {
        self.content.overlay(layout)
    }
}

use crate::ui::ice::renderer::Primitive;

impl<'a, M> From<WorldAnchor<'a, M>> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(anchor: WorldAnchor<'a, M>) -> Element<'a, M, IcedRenderer> { Element::new(anchor) }
}
