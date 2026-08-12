use crate::ui::{graphic, ice::widget::image};

#[derive(Debug)]
pub enum Primitive {
    // Allocation :(
    Group {
        primitives: Vec<Primitive>,
    },
    Image {
        handle: (image::Handle, graphic::Rotation),
        bounds: iced::Rectangle,
        color: vek::Rgba<u8>,
        source_rect: Option<vek::Aabr<f32>>,
    },
    // A vertical gradient
    // TODO: could be combined with rectangle
    Gradient {
        bounds: iced::Rectangle,
        top_linear_color: vek::Rgba<f32>,
        bottom_linear_color: vek::Rgba<f32>,
    },
    Rectangle {
        bounds: iced::Rectangle,
        linear_color: vek::Rgba<f32>,
    },
    Text {
        glyphs: Vec<glyph_brush::SectionGlyph>,
        bounds: iced::Rectangle,
        linear_color: vek::Rgba<f32>,
    },
    Clip {
        bounds: iced::Rectangle,
        offset: vek::Vec2<u32>,
        content: Box<Primitive>,
    },
    // Make content translucent
    Opacity {
        alpha: f32,
        content: Box<Primitive>,
    },
    // Content anchored at a 3D world position (nametags, floaters, aim markers).
    // The renderer projects `pos` with the view-projection matrix and culls the
    // content when it is behind the camera or outside the frustum.
    WorldPos {
        pos: vek::Vec3<f32>,
        // Size of the content in UI coordinates, used for frustum culling.
        dims: vek::Vec2<f32>,
        content: Box<Primitive>,
    },
    Nothing,
}
