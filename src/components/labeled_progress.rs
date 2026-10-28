// SPDX-License-Identifier: Apache-2.0
//! Native determinate progress with split-contrast text; no progress animation.
use super::Appearance;
use crate::animation::Element;
use iced_core::{Length, Pixels, Theme};
use iced_widget::{container, row, space, stack};
pub struct LabeledProgress {
    appearance: Appearance,
    value: f32,
    tone: u32,
    label: String,
    height: f32,
}
impl LabeledProgress {
    pub fn new(appearance: Appearance, value: f32, tone: u32, label: impl Into<String>) -> Self {
        Self {
            appearance,
            value,
            tone,
            label: label.into(),
            height: 18.0,
        }
    }
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into().0;
        self
    }
    pub fn build<'a, M: 'a>(self) -> Element<'a, M> {
        let value = self.value.clamp(0.0, 1.0);
        let tone = self.tone;
        let height = self.height;
        let appearance = self.appearance;
        let face = |filled| {
            container(
                super::text(appearance.font, self.label.clone())
                    .wrapping(iced_core::text::Wrapping::None)
                    .size(if height > 22.0 { 13.333 } else { 10.667 })
                    .style(move |theme: &Theme| {
                        let t = appearance.tokens(theme);
                        iced_widget::text::Style {
                            color: Some(t.contrast(if filled { tone } else { t.border })),
                        }
                    }),
            )
            .padding([0, 8])
            .center_y(height)
            .width(Length::Fill)
            .clip(true)
        };
        let bar: iced_widget::ProgressBar<'a> = iced_widget::progress_bar(0.0..=1.0, value)
            .girth(height)
            .style(move |theme| appearance.tokens(theme).progress(tone)(theme));
        let mut layers = stack![bar, face(false)];
        if value > 0.0 {
            let fraction = (value * 10000.0).round() as u16;
            let filled = container(face(true))
                .width(Length::FillPortion(fraction.max(1)))
                .clip(true);
            layers = layers.push(row![
                filled,
                space().width(Length::FillPortion((10000 - fraction).max(1)))
            ]);
        }
        layers.width(Length::Fill).height(height).into()
    }
}
impl<'a, M: 'a> From<LabeledProgress> for Element<'a, M> {
    fn from(progress: LabeledProgress) -> Self {
        progress.build()
    }
}
