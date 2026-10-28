// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Native split controls and geometry over an app-owned split tree.
use crate::animation::{Element, Renderer};
use iced_core::{Length, Pixels, Size, Theme};
use iced_widget::{
    PaneGrid,
    pane_grid::{self, Axis, Configuration},
};

pub struct SplitLayout<'a, M> {
    inner: PaneGrid<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> SplitLayout<'a, M> {
    pub fn new<T>(
        state: &'a pane_grid::State<T>,
        view: impl Fn(pane_grid::Pane, &'a T, bool) -> pane_grid::Content<'a, M, Theme, Renderer>,
    ) -> Self {
        Self {
            inner: PaneGrid::new(state, view),
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
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(spacing);
        self
    }
    pub fn min_size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.min_size(size);
        self
    }
    pub fn on_resize(
        mut self,
        leeway: impl Into<Pixels>,
        action: impl Fn(pane_grid::ResizeEvent) -> M + 'a,
    ) -> Self {
        self.inner = self.inner.on_resize(leeway, action);
        self
    }
    pub fn build(self) -> PaneGrid<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> From<SplitLayout<'a, M>> for Element<'a, M> {
    fn from(split: SplitLayout<'a, M>) -> Self {
        split.inner.into()
    }
}

/// Borrow a persisted app model; the library never duplicates its panel state.
pub trait SplitModel: Sized {
    type Key: Clone;
    fn panel(&self) -> Option<Self::Key>;
    fn branch(&self) -> Option<(Axis, &[Self], &[f32])>;
}
pub type SplitAddress = (Vec<usize>, usize);
pub fn split_configuration<T: SplitModel>(
    node: &T,
    size: Size,
    gap: f32,
    path: &[usize],
    spec: &mut Vec<SplitAddress>,
) -> Configuration<T::Key> {
    if let Some(key) = node.panel() {
        return Configuration::Pane(key);
    }
    let (axis, children, shares) = node
        .branch()
        .expect("split model must be a panel or branch");
    suffix(axis, children, shares, size, (path, 0), gap, spec)
}
fn suffix<T: SplitModel>(
    axis: Axis,
    children: &[T],
    shares: &[f32],
    size: Size,
    address: (&[usize], usize),
    gap: f32,
    spec: &mut Vec<SplitAddress>,
) -> Configuration<T::Key> {
    let (path, index) = address;
    assert!(
        !children.is_empty() && children.len() == shares.len(),
        "invalid split branch"
    );
    if children.len() == 1 {
        let mut p = path.to_vec();
        p.push(index);
        return split_configuration(&children[0], size, gap, &p, spec);
    }
    let extent = if axis == Axis::Horizontal {
        size.height
    } else {
        size.width
    };
    let available = (extent - gap * (children.len() - 1) as f32).max(1.0);
    let first = (available * shares[0] / shares.iter().sum::<f32>())
        .round()
        .max(1.0);
    let mut a_size = size;
    let mut b_size = size;
    if axis == Axis::Horizontal {
        a_size.height = first;
        b_size.height = (extent - first - gap).max(1.0);
    } else {
        a_size.width = first;
        b_size.width = (extent - first - gap).max(1.0);
    }
    spec.push((path.to_vec(), index));
    let mut p = path.to_vec();
    p.push(index);
    Configuration::Split {
        axis,
        ratio: ((first + gap / 2.0) / extent.max(1.0)).clamp(0.001, 0.999),
        a: Box::new(split_configuration(&children[0], a_size, gap, &p, spec)),
        b: Box::new(suffix(
            axis,
            &children[1..],
            &shares[1..],
            b_size,
            (path, index + 1),
            gap,
            spec,
        )),
    }
}
pub fn split_minimum_width<T: SplitModel>(node: &T, minimum: f32, gap: f32) -> f32 {
    if node.panel().is_some() {
        return minimum;
    }
    let (axis, children, shares) = node.branch().expect("invalid split model");
    if axis == Axis::Horizontal {
        children
            .iter()
            .map(|c| split_minimum_width(c, minimum, gap))
            .fold(0.0, f32::max)
    } else {
        let sum = shares.iter().sum::<f32>();
        children
            .iter()
            .zip(shares)
            .map(|(c, w)| split_minimum_width(c, minimum, gap) * sum / w)
            .fold(0.0, f32::max)
            + gap * children.len().saturating_sub(1) as f32
    }
}
pub fn resize_split_shares(shares: &mut [f32], index: usize, extent: f32, ratio: f32, gap: f32) {
    if index + 1 >= shares.len() {
        return;
    }
    let tail: f32 = shares[index..].iter().sum();
    let old_rest: f32 = shares[index + 1..].iter().sum();
    if old_rest <= 0.0 {
        return;
    }
    let pixels = (extent * ratio - gap / 2.0).round();
    let available = (extent - gap * (shares.len() - index - 1) as f32).max(1.0);
    let fraction = (pixels / available).clamp(0.001, 0.999);
    shares[index] = tail * fraction;
    for w in &mut shares[index + 1..] {
        *w *= tail * (1.0 - fraction) / old_rest;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockEdge {
    Top,
    Left,
    Right,
    Bottom,
}
impl DockEdge {
    pub fn at(bounds: iced_core::Rectangle, point: iced_core::Point) -> Option<Self> {
        if !bounds.contains(point) || bounds.width <= 0.0 || bounds.height <= 0.0 {
            return None;
        }
        let x = (point.x - bounds.x) / bounds.width;
        let y = (point.y - bounds.y) / bounds.height;
        let (distance, edge) = [
            (y, Self::Top),
            (x, Self::Left),
            (1.0 - x, Self::Right),
            (1.0 - y, Self::Bottom),
        ]
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))?;
        (distance <= 0.25).then_some(edge)
    }
}
