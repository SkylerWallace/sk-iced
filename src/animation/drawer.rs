// SPDX-License-Identifier: Apache-2.0
//! Interruptible drawer controller with caller-owned geometry and policy.
use std::time::{Duration, Instant};
#[derive(Debug)]
pub struct Drawer {
    collapsed: f32,
    expanded: f32,
    pub width: f32,
    pub open: bool,
    from: f32,
    started: Instant,
    duration: Duration,
    pub animations: bool,
}
impl Drawer {
    pub fn new(collapsed: f32, expanded: f32, animations: bool) -> Self {
        assert!(collapsed.is_finite() && expanded.is_finite() && expanded > collapsed);
        Self {
            collapsed,
            expanded,
            width: collapsed,
            open: false,
            from: collapsed,
            started: Instant::now(),
            duration: Duration::ZERO,
            animations,
        }
    }
    pub fn set(&mut self, open: bool) {
        self.advance();
        self.from = self.width;
        self.open = open;
        self.started = Instant::now();
        self.duration = if self.animations {
            Duration::from_millis(if open { 200 } else { 150 })
        } else {
            Duration::ZERO
        };
        if !self.animations {
            self.width = if open { self.expanded } else { self.collapsed };
        }
    }
    pub fn animating(&self) -> bool {
        self.width
            != if self.open {
                self.expanded
            } else {
                self.collapsed
            }
    }
    pub fn set_motion(&mut self, policy: crate::motion::Policy) {
        self.animations = policy.enabled();
        if !self.animations {
            self.duration = Duration::ZERO;
            self.width = if self.open {
                self.expanded
            } else {
                self.collapsed
            };
        }
    }
    pub fn advance(&mut self) {
        let progress = if self.duration.is_zero() {
            1.0
        } else {
            (self.started.elapsed().as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        };
        let target = if self.open {
            self.expanded
        } else {
            self.collapsed
        };
        self.width = if progress >= 1.0 {
            target
        } else {
            self.from + (target - self.from) * eased(progress, self.open)
        };
    }
    pub fn morph(&self) -> usize {
        (((self.width - self.collapsed) / (self.expanded - self.collapsed) * 20.0).round() as usize)
            .min(20)
    }
}
pub fn eased(progress: f32, entering: bool) -> f32 {
    let (x1, x2) = if entering { (0.0, 0.2) } else { (0.4, 1.0) };
    let cubic = |t: f32, a: f32, b: f32| {
        3.0 * (1.0 - t).powi(2) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
    };
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..16 {
        let mid = (low + high) / 2.0;
        if cubic(mid, x1, x2) < progress {
            low = mid
        } else {
            high = mid
        }
    }
    cubic((low + high) / 2.0, 0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversing_retains_pose_and_disabling_snaps_without_queued_work() {
        let mut drawer = Drawer::new(56.0, 208.0, true);
        drawer.set(true);
        drawer.started = Instant::now() - Duration::from_millis(100);
        drawer.advance();
        let width = drawer.width;
        assert!(width > 56.0 && width < 208.0);
        drawer.set(false);
        assert!((drawer.width - width).abs() < 1.0);
        drawer.set_motion(crate::motion::Policy::DISABLED);
        assert_eq!(drawer.width, 56.0);
        assert!(!drawer.animating());
        drawer.set(true);
        assert_eq!(drawer.width, 208.0);
        assert!(!drawer.animating());
        assert_eq!(drawer.morph(), 20);
    }
}
