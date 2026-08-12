//! Image-based slider (port of the Conrod `ImageSlider`).
use super::image;
use crate::ui::ice::{IcedRenderer, renderer::Primitive};
use iced::{
    Element, Event, Hasher, Layout, Length, Point, Rectangle, Size, Widget, event, layout, mouse,
};
use num::{Float, Integer, Num, NumCast};
use std::hash::Hash;
use vek::*;

/// Marker for a continuous slider.
pub enum Continuous {}
/// Marker for a discrete slider.
pub enum Discrete {}

/// Converts a fractional position in `[0, 1]` to a value in `[min, max]`.
pub trait ValueFromPercent<T> {
    fn value_from_percent(percent: f32, min: T, max: T) -> T;
}

/// Linear interpolation of `percent` between `min` and `max`.
pub fn lerp(percent: f32, min: f32, max: f32) -> f32 { min + (max - min) * percent }

/// Inverse: position of `value` between `min` and `max` in `[0, 1]`.
pub fn unlerp(value: f32, min: f32, max: f32) -> f32 {
    if max == min {
        0.0
    } else {
        (value - min) / (max - min)
    }
}

/// The rect of the slider handle for a value (port of the Conrod logic).
pub fn slider_rect(
    bounds: Rectangle,
    value: f32,
    min: f32,
    max: f32,
    skew: f32,
    is_horizontal: bool,
    slider_length: f32,
    start_pad: f32,
    end_pad: f32,
) -> Rectangle {
    let value_perc = unlerp(value, min, max);
    let unskewed_perc = value_perc.powf(1.0 / skew);
    let pos = lerp(unskewed_perc, 0.0, 1.0);
    if is_horizontal {
        let x = lerp(pos, bounds.x + start_pad, bounds.x + bounds.width - end_pad)
            .min(bounds.x + bounds.width - slider_length);
        Rectangle {
            x,
            y: bounds.y,
            width: slider_length,
            height: bounds.height,
        }
    } else {
        let y = lerp(
            pos,
            bounds.y + start_pad,
            bounds.y + bounds.height - end_pad,
        )
        .min(bounds.y + bounds.height - slider_length);
        Rectangle {
            x: bounds.x,
            y,
            width: bounds.width,
            height: slider_length,
        }
    }
}

/// Computes the value from a mouse offset along the track (port of the Conrod
/// drag logic).
pub fn value_from_offset(offset: f32, track_length: f32, min: f32, max: f32, skew: f32) -> f32 {
    let perc = offset.clamp(0.0, track_length) / track_length;
    let skewed_perc = perc.powf(skew);
    lerp(skewed_perc, min, max)
}

/// Per-widget state for drag tracking.
#[derive(Debug, Default)]
pub struct State {
    dragging: bool,
}

pub struct ImageSlider<'a, T, K, M> {
    value: T,
    min: T,
    max: T,
    // If `value > soft_max` the slider is displayed at `soft_max` along with a
    // faded ghost slider at `value`.
    soft_max: T,
    /// Higher skew amounts (above 1.0) weigh lower values.
    skew: f32,
    track_image: image::Handle,
    slider_image: image::Handle,
    track_color: Option<Rgba<u8>>,
    slider_color: Option<Rgba<u8>>,
    // Padding on the ends of the track constraining the slider.
    pad_track: (f32, f32),
    slider_length: Option<f32>,
    on_change: Box<dyn Fn(T) -> M + 'a>,
    width: Length,
    height: Length,
    state: &'a mut State,
    kind: std::marker::PhantomData<K>,
}

impl<'a, T, K, M> ImageSlider<'a, T, K, M>
where
    T: Copy,
{
    fn new(
        value: T,
        min: T,
        max: T,
        slider_image: image::Handle,
        track_image: image::Handle,
        on_change: impl Fn(T) -> M + 'a,
        state: &'a mut State,
    ) -> Self {
        Self {
            value,
            min,
            soft_max: max,
            max,
            skew: 1.0,
            track_image,
            slider_image,
            track_color: None,
            slider_color: None,
            pad_track: (0.0, 0.0),
            slider_length: None,
            on_change: Box::new(on_change),
            width: Length::Shrink,
            height: Length::Shrink,
            state,
            kind: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn skew(mut self, skew: f32) -> Self {
        self.skew = skew;
        self
    }

    #[must_use]
    pub fn soft_max(mut self, soft_max: T) -> Self {
        self.soft_max = soft_max;
        self
    }

    #[must_use]
    pub fn pad_track(mut self, pad: (f32, f32)) -> Self {
        self.pad_track = pad;
        self
    }

    #[must_use]
    pub fn track_color(mut self, color: Rgba<u8>) -> Self {
        self.track_color = Some(color);
        self
    }

    #[must_use]
    pub fn slider_color(mut self, color: Rgba<u8>) -> Self {
        self.slider_color = Some(color);
        self
    }

    #[must_use]
    pub fn slider_length(mut self, length: f32) -> Self {
        self.slider_length = Some(length);
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

impl<'a, T, M> ImageSlider<'a, T, Continuous, M>
where
    T: Copy,
{
    pub fn continuous(
        value: T,
        min: T,
        max: T,
        slider_image: image::Handle,
        track_image: image::Handle,
        on_change: impl Fn(T) -> M + 'a,
        state: &'a mut State,
    ) -> Self {
        ImageSlider::new(value, min, max, slider_image, track_image, on_change, state)
    }
}

impl<'a, T, M> ImageSlider<'a, T, Discrete, M>
where
    T: Copy,
{
    pub fn discrete(
        value: T,
        min: T,
        max: T,
        slider_image: image::Handle,
        track_image: image::Handle,
        on_change: impl Fn(T) -> M + 'a,
        state: &'a mut State,
    ) -> Self {
        ImageSlider::new(value, min, max, slider_image, track_image, on_change, state)
    }
}

impl<T: Float> ValueFromPercent<T> for Continuous {
    fn value_from_percent(percent: f32, min: T, max: T) -> T {
        NumCast::from(lerp(percent, min.to_f32().unwrap(), max.to_f32().unwrap())).unwrap()
    }
}

impl<T: Integer + NumCast> ValueFromPercent<T> for Discrete {
    fn value_from_percent(percent: f32, min: T, max: T) -> T {
        NumCast::from(lerp(percent, min.to_f32().unwrap(), max.to_f32().unwrap()).round()).unwrap()
    }
}

impl<T, K, M> Widget<M, IcedRenderer> for ImageSlider<'_, T, K, M>
where
    T: NumCast + Num + Copy + PartialOrd,
    K: ValueFromPercent<T>,
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
        _defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();
        let is_horizontal = bounds.width > bounds.height;
        let (start_pad, end_pad) = self.pad_track;

        // Track.
        let track_rect = if is_horizontal {
            let h = self
                .slider_length
                .map_or(bounds.height / 3.0, |l| l.min(bounds.height));
            Rectangle {
                x: bounds.x,
                y: bounds.y + (bounds.height - h) / 2.0,
                width: bounds.width,
                height: h,
            }
        } else {
            let w = self
                .slider_length
                .map_or(bounds.width / 3.0, |l| l.min(bounds.width));
            Rectangle {
                x: bounds.x + (bounds.width - w) / 2.0,
                y: bounds.y,
                width: w,
                height: bounds.height,
            }
        };
        let mut primitives = vec![renderer.draw_image(
            self.track_image,
            track_rect,
            self.track_color.unwrap_or(Rgba::broadcast(255)),
        )];

        let slider_length = self.slider_length.unwrap_or(if is_horizontal {
            bounds.width / 10.0
        } else {
            bounds.height / 10.0
        });

        let over_soft_max = self.value > self.soft_max;
        let fade = if over_soft_max { 0.5 } else { 1.0 };

        // Main slider.
        let (min_f, max_f) = (self.min.to_f32().unwrap(), self.max.to_f32().unwrap());
        let rect = slider_rect(
            bounds,
            self.value.to_f32().unwrap(),
            min_f,
            max_f,
            self.skew,
            is_horizontal,
            slider_length,
            start_pad,
            end_pad,
        );
        let color = self
            .slider_color
            .map_or(Rgba::new(255, 255, 255, (fade * 255.0) as u8), |c| {
                Rgba::new(c.r, c.g, c.b, (c.a as f32 * fade) as u8)
            });
        primitives.push(renderer.draw_image(self.slider_image, rect, color));

        // Ghost slider at soft_max.
        if over_soft_max {
            let soft_rect = slider_rect(
                bounds,
                self.soft_max.to_f32().unwrap(),
                min_f,
                max_f,
                self.skew,
                is_horizontal,
                slider_length,
                start_pad,
                end_pad,
            );
            primitives.push(renderer.draw_image(
                self.slider_image,
                soft_rect,
                self.slider_color.unwrap_or(Rgba::broadcast(255)),
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

        self.width.hash(state);
        self.height.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn iced::Clipboard,
        messages: &mut Vec<M>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let is_horizontal = bounds.width > bounds.height;
        let (start_pad, end_pad) = self.pad_track;

        let value_from_cursor = |cursor: Point| {
            let track_length = if is_horizontal {
                bounds.width - start_pad - end_pad
            } else {
                bounds.height - start_pad - end_pad
            };
            let offset = if is_horizontal {
                cursor.x - bounds.x - start_pad
            } else {
                cursor.y - bounds.y - start_pad
            };
            let perc = (offset.clamp(0.0, track_length) / track_length).powf(self.skew);
            K::value_from_percent(perc, self.min, self.max)
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if bounds.contains(cursor_position) {
                    self.state.dragging = true;
                    let new_value = value_from_cursor(cursor_position);
                    if new_value != self.value {
                        self.value = new_value;
                        messages.push((self.on_change)(self.value));
                    }
                }
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.state.dragging {
                    let new_value = value_from_cursor(cursor_position);
                    if new_value != self.value {
                        self.value = new_value;
                        messages.push((self.on_change)(self.value));
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.state.dragging = false;
            },
            _ => {},
        }
        event::Status::Ignored
    }
}

impl<'a, T, K, M> From<ImageSlider<'a, T, K, M>> for Element<'a, M, IcedRenderer>
where
    T: NumCast + Num + Copy + PartialOrd + 'a,
    K: ValueFromPercent<T> + 'a,
    M: 'a,
{
    fn from(slider: ImageSlider<'a, T, K, M>) -> Element<'a, M, IcedRenderer> {
        Element::new(slider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_from_percent_linear() {
        assert_eq!(lerp(0.0, 0.0, 100.0), 0.0);
        assert_eq!(lerp(0.5, 0.0, 100.0), 50.0);
        assert_eq!(lerp(1.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn discrete_rounds() {
        let v = <Discrete as ValueFromPercent<u32>>::value_from_percent(0.333, 0, 100);
        assert_eq!(v, 33);
        let v = <Discrete as ValueFromPercent<u32>>::value_from_percent(0.999, 0, 100);
        assert_eq!(v, 100);
    }

    #[test]
    fn continuous_interpolates() {
        let v = <Continuous as ValueFromPercent<f32>>::value_from_percent(0.25, 0.0, 8.0);
        assert!((v - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn offset_maps_to_value() {
        let v = value_from_offset(50.0, 100.0, 0.0, 10.0, 1.0);
        assert!((v - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn offset_clamped_to_track() {
        let v = value_from_offset(-50.0, 100.0, 0.0, 10.0, 1.0);
        assert_eq!(v, 0.0);
        let v = value_from_offset(150.0, 100.0, 0.0, 10.0, 1.0);
        assert_eq!(v, 10.0);
    }

    #[test]
    fn skew_weighs_lower_values() {
        // With skew 2.0, half the track maps to ~29% of the range.
        let v = value_from_offset(50.0, 100.0, 0.0, 10.0, 2.0);
        assert!((v - 2.5).abs() < 0.01, "skewed value: {v}");
    }

    #[test]
    fn slider_rect_positions() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        };
        let at_min = slider_rect(bounds, 0.0, 0.0, 100.0, 1.0, true, 10.0, 0.0, 0.0);
        assert_eq!(at_min.x, 0.0);
        let at_mid = slider_rect(bounds, 50.0, 0.0, 100.0, 1.0, true, 10.0, 0.0, 0.0);
        assert_eq!(at_mid.x, 50.0);
        let at_max = slider_rect(bounds, 100.0, 0.0, 100.0, 1.0, true, 10.0, 0.0, 0.0);
        assert_eq!(at_max.x, 90.0, "handle right edge sits at the track end");
    }

    #[test]
    fn slider_rect_vertical() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 100.0,
        };
        let rect = slider_rect(bounds, 50.0, 0.0, 100.0, 1.0, false, 10.0, 0.0, 0.0);
        assert_eq!(rect.y, 50.0);
        assert_eq!(rect.width, 20.0);
        assert_eq!(rect.height, 10.0);
    }

    #[test]
    fn slider_rect_respects_padding() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        };
        let at_max = slider_rect(bounds, 100.0, 0.0, 100.0, 1.0, true, 10.0, 5.0, 10.0);
        assert_eq!(at_max.x, 90.0, "end pad constrains the handle start");
    }
}
