// SPDX-License-Identifier: Apache-2.0
//! Reusable dialog command policy and native backdrops. Apps retain focus IDs,
//! routing priority and busy-state guards for their controls and operations.
use crate::animation::{Element, Renderer};
use iced_core::{
    Color, Length, Padding, Theme,
    keyboard::{Key, key::Named},
};
use iced_widget::{Container, container};
#[derive(Clone)]
pub struct DialogCommands<M> {
    default: Option<M>,
    cancel: Option<M>,
    busy: bool,
}
impl<M: Clone> DialogCommands<M> {
    pub fn new() -> Self {
        Self {
            default: None,
            cancel: None,
            busy: false,
        }
    }
    pub fn default_action(mut self, action: M) -> Self {
        self.default = Some(action);
        self
    }
    pub fn cancel_action(mut self, action: M) -> Self {
        self.cancel = Some(action);
        self
    }
    /// Blocks the default acceptance command. Cancel remains independently usable.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }
    pub fn action_named(&self, key: Named) -> Option<M> {
        match key {
            Named::Enter if !self.busy => self.default.clone(),
            Named::Escape => self.cancel.clone(),
            _ => None,
        }
    }
    pub fn action(&self, key: &Key) -> Option<M> {
        if let Key::Named(key) = key {
            self.action_named(*key)
        } else {
            None
        }
    }
}
impl<M: Clone> Default for DialogCommands<M> {
    fn default() -> Self {
        Self::new()
    }
}
pub struct DialogBackdrop<'a, M> {
    inner: Container<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> DialogBackdrop<'a, M> {
    pub fn new(card: impl Into<Element<'a, M>>) -> Self {
        Self::custom(container(card).center(Length::Fill))
            .scrim(Color::from_rgba(0.0, 0.0, 0.0, 0.55))
    }
    pub fn custom(inner: Container<'a, M, Theme, Renderer>) -> Self {
        Self { inner }
    }
    pub fn scrim(mut self, color: Color) -> Self {
        self.inner = self.inner.style(move |_| container::Style {
            background: Some(color.into()),
            ..Default::default()
        });
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }
    pub fn build(self) -> Container<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> From<DialogBackdrop<'a, M>> for Element<'a, M> {
    fn from(backdrop: DialogBackdrop<'a, M>) -> Self {
        backdrop.inner.into()
    }
}
