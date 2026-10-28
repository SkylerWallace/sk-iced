// SPDX-License-Identifier: Apache-2.0
//! Selectable software-rendered text. Content, scroll routing and copy actions
//! remain app-owned; perform_read_only rejects edits while preserving selection.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::{Font, Length, Pixels};
use iced_widget::{stack, text_editor};
pub enum ReadOnlyAction {
    Applied,
    IgnoredEdit,
    Scroll(i32),
}
pub fn perform_read_only(
    content: &mut text_editor::Content<Renderer>,
    action: text_editor::Action,
) -> ReadOnlyAction {
    if let text_editor::Action::Scroll { lines } = action {
        return ReadOnlyAction::Scroll(lines);
    }
    if action.is_edit() {
        ReadOnlyAction::IgnoredEdit
    } else {
        content.perform(action);
        ReadOnlyAction::Applied
    }
}
pub struct ReadOnlyText<'a, M> {
    appearance: Appearance,
    content: &'a text_editor::Content<Renderer>,
    font: Font,
    size: f32,
    height: Length,
    action: Option<Box<dyn Fn(text_editor::Action) -> M + 'a>>,
}
impl<'a, M: Clone + 'a> ReadOnlyText<'a, M> {
    pub fn new(appearance: Appearance, content: &'a text_editor::Content<Renderer>) -> Self {
        Self {
            appearance,
            content,
            font: Font::MONOSPACE,
            size: crate::design::text::BODY,
            height: Length::Fill,
            action: None,
        }
    }
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn on_action(mut self, action: impl Fn(text_editor::Action) -> M + 'a) -> Self {
        self.action = Some(Box::new(action));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let appearance = self.appearance;
        let mut editor = iced_widget::text_editor(self.content)
            .font(self.font)
            .size(self.size)
            .height(self.height)
            .padding(0)
            .style(move |theme, _| {
                let t = appearance.tokens(theme);
                text_editor::Style {
                    background: iced_core::Color {
                        a: 0.0,
                        ..color(t.surface)
                    }
                    .into(),
                    border: iced_core::Border::default(),
                    placeholder: color(t.secondary),
                    value: iced_core::Color {
                        a: 0.0,
                        ..color(t.text)
                    },
                    selection: iced_core::Color {
                        a: t.opacity_selected as f32,
                        ..color(t.accent)
                    },
                }
            });
        if let Some(action) = self.action {
            editor = editor.on_action(action);
        }
        // Keep native text above the editor's software-renderer glyph limitation,
        // with the native editor providing selection and clipboard hit testing.
        stack![
            super::text(self.font, self.content.text())
                .size(self.size)
                .width(Length::Fill),
            editor
        ]
        .into()
    }
}
impl<'a, M: Clone + 'a> From<ReadOnlyText<'a, M>> for Element<'a, M> {
    fn from(view: ReadOnlyText<'a, M>) -> Self {
        view.build()
    }
}
