// SPDX-License-Identifier: MIT AND Apache-2.0
//! Native data layout builders. Models, stable item IDs and sorting remain app-owned.
use crate::animation::{Element, Renderer};
use iced_core::{Length, Padding, Pixels, Theme, alignment};
use iced_widget::{Column, Row};

/// A native row: no new widget state or interception of child actions.
pub struct TableRow<'a, M> {
    inner: Row<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> TableRow<'a, M> {
    pub fn new() -> Self {
        Self {
            inner: Row::new().align_y(alignment::Vertical::Center),
        }
    }
    pub fn custom(inner: Row<'a, M, Theme, Renderer>) -> Self {
        Self { inner }
    }
    pub fn push(mut self, cell: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.push(cell);
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(spacing);
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
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.inner = self.inner.align_y(alignment);
        self
    }
    pub fn build(self) -> Row<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> Default for TableRow<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a, M: 'a> From<TableRow<'a, M>> for Element<'a, M> {
    fn from(row: TableRow<'a, M>) -> Self {
        row.inner.into()
    }
}

/// Header/body composition, deliberately without a second scroll owner.
pub struct DataTable<'a, M> {
    inner: Column<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> DataTable<'a, M> {
    pub fn new(header: impl Into<Element<'a, M>>, body: impl Into<Element<'a, M>>) -> Self {
        Self {
            inner: Column::new().push(header).push(body),
        }
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }
    pub fn build(self) -> Column<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> From<DataTable<'a, M>> for Element<'a, M> {
    fn from(table: DataTable<'a, M>) -> Self {
        table.inner.into()
    }
}

/// Ordered selection over stable IDs. Reordering the visible slice does not
/// change selected identities. Callers retain ownership/persistence of all state.
pub fn select<K: Ord + Clone>(
    selected: &mut std::collections::BTreeSet<K>,
    anchor: &mut Option<K>,
    order: &[K],
    item: K,
    extend: bool,
    toggle: bool,
) {
    if extend {
        let start = anchor
            .as_ref()
            .and_then(|a| order.iter().position(|v| v == a));
        let end = order.iter().position(|v| *v == item);
        if !toggle {
            selected.clear();
        }
        if let (Some(a), Some(b)) = (start, end) {
            selected.extend(order[a.min(b)..=a.max(b)].iter().cloned());
        } else {
            selected.insert(item.clone());
        }
    } else if toggle {
        if !selected.remove(&item) {
            selected.insert(item.clone());
        }
    } else {
        selected.clear();
        selected.insert(item.clone());
    }
    *anchor = if selected.contains(&item) {
        Some(item)
    } else {
        selected.iter().next().cloned()
    };
}

/// Bounded visible-index navigation, usable without allocating a selection model.
pub fn adjacent(len: usize, current: Option<usize>, direction: i8) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(match current {
            Some(i) if direction < 0 => i.saturating_sub(1).min(len - 1),
            Some(i) => i.saturating_add(1).min(len - 1),
            None => 0,
        })
    }
}

/// Ordered stable column keys and visibility, independent of app persistence and
/// sort semantics. Hidden columns keep their position when restored.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumns<K> {
    pub order: Vec<K>,
    pub hidden: Vec<K>,
}
impl<K: Clone + PartialEq> TableColumns<K> {
    pub fn new(order: impl IntoIterator<Item = K>) -> Self {
        Self {
            order: order.into_iter().collect(),
            hidden: Vec::new(),
        }
    }
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = K>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }
    pub fn visible(&self) -> Vec<K> {
        self.order
            .iter()
            .filter(|key| !self.hidden.contains(key))
            .cloned()
            .collect()
    }
    pub fn toggle(&mut self, key: K) {
        if self.hidden.contains(&key) {
            self.hidden.retain(|k| *k != key);
        } else {
            self.hidden.push(key);
        }
    }
    pub fn move_to(&mut self, from: K, to: K) {
        if from == to || !self.order.contains(&from) {
            return;
        }
        let at = self
            .order
            .iter()
            .position(|k| *k == to)
            .unwrap_or(self.order.len().saturating_sub(1));
        self.order.retain(|k| *k != from);
        self.order.insert(at.min(self.order.len()), from);
    }
}
