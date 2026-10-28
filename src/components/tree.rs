// SPDX-License-Identifier: MIT AND Apache-2.0
//! Flattened tree presentation. Apps enumerate/load children and own expansion,
//! stable keys, selection and context actions. No filesystem dependency.
use super::{Appearance, VirtualList};
use crate::animation::Element;
use iced_core::Length;
use iced_widget::{Row, Space};

/// Virtualized tree over app-owned visible rows. Loading/error/empty rows are
/// ordinary row content; collapsing never removes an app's hidden child state.
pub struct TreeView<'a, M> {
    viewport: VirtualList<'a, M>,
}
impl<'a, M: 'a> TreeView<'a, M> {
    pub fn new(
        appearance: Appearance,
        len: usize,
        identity: u64,
        row: impl Fn(usize) -> Element<'a, M> + 'a,
    ) -> Self {
        Self {
            viewport: VirtualList::new(appearance, len, identity, row),
        }
    }
    pub fn row_height(mut self, height: f32) -> Self {
        self.viewport = self.viewport.row_height(height);
        self
    }
    pub fn reveal(mut self, request: Option<(u64, usize)>) -> Self {
        self.viewport = self.viewport.reveal(request);
        self
    }
    pub fn build(self) -> Element<'a, M> {
        self.viewport.build()
    }
}
impl<'a, M: 'a> From<TreeView<'a, M>> for Element<'a, M> {
    fn from(tree: TreeView<'a, M>) -> Self {
        tree.build()
    }
}

/// Disclosure and content remain separate action surfaces. This returns a native
/// Row so focus, child state, operations and input routing are unchanged.
pub struct TreeRow<'a, M> {
    depth: usize,
    indent: f32,
    max_depth: usize,
    height: Length,
    disclosure: Element<'a, M>,
    content: Element<'a, M>,
}
impl<'a, M: 'a> TreeRow<'a, M> {
    pub fn new(disclosure: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            depth: 0,
            indent: 14.0,
            max_depth: 12,
            height: Length::Shrink,
            disclosure: disclosure.into(),
            content: content.into(),
        }
    }
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent.max(0.0);
        self
    }
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn build(self) -> Row<'a, M, iced_core::Theme, crate::animation::Renderer> {
        Row::new()
            .push(Space::new().width(self.depth.min(self.max_depth) as f32 * self.indent))
            .push(self.disclosure)
            .push(self.content)
            .height(self.height)
    }
}
impl<'a, M: 'a> From<TreeRow<'a, M>> for Element<'a, M> {
    fn from(row: TreeRow<'a, M>) -> Self {
        row.build().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNavigation {
    Select(usize),
    Toggle(usize),
    None,
}
/// Native tree arrow rules over visible hierarchy metadata. The app resolves
/// parents/children by stable keys and decides how to load or toggle nodes.
pub fn tree_navigation(
    len: usize,
    current: Option<usize>,
    direction: i8,
    expanded: bool,
    parent: Option<usize>,
) -> TreeNavigation {
    use TreeNavigation::*;
    if len == 0 {
        return None;
    }
    if let Some(index) = current.filter(|i| *i < len) {
        if direction == 2 && !expanded || direction == -2 && expanded {
            return Toggle(index);
        }
        if direction == -2 {
            return parent.filter(|i| *i < len).map_or(None, Select);
        }
    }
    super::adjacent(len, current, direction).map_or(None, Select)
}
