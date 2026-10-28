// SPDX-License-Identifier: Apache-2.0
//! Native navigation and toolbar composition; routes and commands remain local.
use crate::animation::{Element, Renderer};
use iced_core::{Length, Padding, Pixels, Theme, alignment};
use iced_widget::{Row, column, row};

pub struct Toolbar<'a, M> {
    inner: Row<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> Toolbar<'a, M> {
    pub fn new() -> Self {
        Self { inner: Row::new() }
    }
    pub fn push(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.push(content);
        self
    }
    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(value);
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn align_y(mut self, value: impl Into<alignment::Vertical>) -> Self {
        self.inner = self.inner.align_y(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.height(value);
        self
    }
    pub fn build(self) -> Row<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> Default for Toolbar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a, M: 'a> From<Toolbar<'a, M>> for Element<'a, M> {
    fn from(toolbar: Toolbar<'a, M>) -> Self {
        toolbar.inner.into()
    }
}

/// The icon/badge surface and caption keep separate actions and styles.
pub struct NavigationItem<'a, M> {
    inner: Row<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> NavigationItem<'a, M> {
    pub fn new(icon: impl Into<Element<'a, M>>) -> Self {
        Self {
            inner: Row::new().push(icon),
        }
    }
    pub fn caption(mut self, caption: impl Into<Element<'a, M>>, expanded: bool) -> Self {
        if expanded {
            self.inner = self.inner.push(caption);
        }
        self
    }
    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(value);
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.height(value);
        self
    }
    pub fn build(self) -> Row<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> From<NavigationItem<'a, M>> for Element<'a, M> {
    fn from(item: NavigationItem<'a, M>) -> Self {
        item.inner.into()
    }
}

/// A sidebar/body row. Header and footer slots are optional, so the default
/// path preserves the native row topology and widget-local state.
pub struct AppShell<'a, M> {
    sidebar: Element<'a, M>,
    body: Element<'a, M>,
    header: Option<Element<'a, M>>,
    footer: Option<Element<'a, M>>,
}
impl<'a, M: 'a> AppShell<'a, M> {
    pub fn new(sidebar: impl Into<Element<'a, M>>, body: impl Into<Element<'a, M>>) -> Self {
        Self {
            sidebar: sidebar.into(),
            body: body.into(),
            header: None,
            footer: None,
        }
    }
    pub fn header(mut self, header: impl Into<Element<'a, M>>) -> Self {
        self.header = Some(header.into());
        self
    }
    pub fn footer(mut self, footer: impl Into<Element<'a, M>>) -> Self {
        self.footer = Some(footer.into());
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let body = row![self.sidebar, self.body]
            .width(Length::Fill)
            .height(Length::Fill);
        if self.header.is_none() && self.footer.is_none() {
            return body.into();
        }
        let mut parts = column![];
        if let Some(header) = self.header {
            parts = parts.push(header);
        }
        parts = parts.push(body);
        if let Some(footer) = self.footer {
            parts = parts.push(footer);
        }
        parts.width(Length::Fill).height(Length::Fill).into()
    }
}
impl<'a, M: 'a> From<AppShell<'a, M>> for Element<'a, M> {
    fn from(shell: AppShell<'a, M>) -> Self {
        shell.build()
    }
}

/// Immediate logical dismissal with the existing paint-only closing reveal.
/// Optional backdrop and foreground slots preserve native stack ordering.
pub struct NavigationDrawer<'a, M> {
    base: Element<'a, M>,
    drawer: Element<'a, M>,
    backdrop: Option<Element<'a, M>>,
    foreground: Option<Element<'a, M>>,
}
impl<'a, M: 'a> NavigationDrawer<'a, M> {
    pub fn new(
        base: impl Into<Element<'a, M>>,
        open: bool,
        policy: crate::motion::Policy,
        drawer: impl Fn() -> Element<'a, M> + 'a,
    ) -> Self {
        Self {
            base: base.into(),
            drawer: crate::animation::reveal::horizontal(open, policy, drawer),
            backdrop: None,
            foreground: None,
        }
    }
    pub fn backdrop(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.backdrop = Some(content.into());
        self
    }
    pub fn foreground(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.foreground = Some(content.into());
        self
    }
    pub fn build(self) -> iced_widget::Stack<'a, M, Theme, Renderer> {
        let mut layers = iced_widget::Stack::new().push(self.base);
        if let Some(backdrop) = self.backdrop {
            layers = layers.push(backdrop);
        }
        layers = layers.push(self.drawer);
        if let Some(foreground) = self.foreground {
            layers = layers.push(foreground);
        }
        layers
    }
}
impl<'a, M: 'a> From<NavigationDrawer<'a, M>> for Element<'a, M> {
    fn from(drawer: NavigationDrawer<'a, M>) -> Self {
        drawer.build().into()
    }
}
