//! Shared motion policy and allocation-free animation state for custom widgets.
//! Keep state in the widget Tree, sample on RedrawRequested, and call
//! request_frame only while active. Operations and hit testing must never wait
//! for a visual transition. No global animation timer or background polling.
use std::time::{Duration, Instant};

pub const QUICK: Duration = Duration::from_millis(100);
pub const STANDARD: Duration = Duration::from_millis(150);
pub const FEEDBACK: Duration = Duration::from_millis(220);

pub use crate::motion::Policy;
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseOut,
}
impl Easing {
    fn sample(self, progress: f32) -> f32 {
        match self {
            Self::Linear => progress,
            Self::EaseOut => 1.0 - (1.0 - progress).powi(3),
        }
    }
}

/// Retargetable scalar used for reveal, opacity, displacement, or rotation.
#[derive(Debug, Clone)]
pub struct Tween {
    from: f32,
    target: f32,
    start: Instant,
    duration: Duration,
    easing: Easing,
}
impl Tween {
    pub fn new(value: f32, now: Instant) -> Self {
        assert!(value.is_finite());
        Self {
            from: value,
            target: value,
            start: now,
            duration: Duration::ZERO,
            easing: Easing::EaseOut,
        }
    }
    pub fn value(&self, now: Instant, policy: Policy) -> f32 {
        if !self.active(now, policy) {
            return self.target;
        }
        let progress =
            now.saturating_duration_since(self.start).as_secs_f32() / self.duration.as_secs_f32();
        self.from + (self.target - self.from) * self.easing.sample(progress.clamp(0.0, 1.0))
    }
    pub fn active(&self, now: Instant, policy: Policy) -> bool {
        policy.enabled()
            && self.from != self.target
            && now.saturating_duration_since(self.start) < self.duration
    }
    pub fn retarget(
        &mut self,
        target: f32,
        now: Instant,
        duration: Duration,
        easing: Easing,
        policy: Policy,
    ) {
        assert!(target.is_finite());
        if self.target == target {
            self.settle_if_disabled(policy);
            return;
        }
        self.from = self.value(now, policy);
        self.target = target;
        self.start = now;
        self.duration = duration;
        self.easing = easing;
        self.settle_if_disabled(policy);
    }
    /// Persist the final value on policy change so re-enabling cannot resume motion.
    pub fn settle_if_disabled(&mut self, policy: Policy) {
        if !policy.enabled() {
            self.from = self.target;
            self.duration = Duration::ZERO;
        }
    }
    pub fn request_frame<M>(
        &self,
        now: Instant,
        policy: Policy,
        shell: &mut iced_core::Shell<'_, M>,
    ) {
        if self.active(now, policy) {
            shell.request_redraw();
        }
    }
}

/// One bounded click ripple. Drawing stays clipped to the owning button.
#[derive(Debug, Clone)]
pub struct Ripple {
    pub origin: iced_core::Point,
    radius: f32,
    progress: Tween,
}
impl Ripple {
    pub fn new(
        origin: iced_core::Point,
        size: iced_core::Size,
        now: Instant,
        policy: Policy,
    ) -> Self {
        let origin = iced_core::Point::new(
            origin.x.clamp(0.0, size.width),
            origin.y.clamp(0.0, size.height),
        );
        let radius = origin
            .x
            .max(size.width - origin.x)
            .hypot(origin.y.max(size.height - origin.y));
        let mut progress = Tween::new(0.0, now);
        progress.retarget(1.0, now, FEEDBACK, Easing::EaseOut, policy);
        Self {
            origin,
            radius,
            progress,
        }
    }
    /// None when finished or disabled; no lingering highlight under disabled motion.
    pub fn sample(&self, now: Instant, policy: Policy) -> Option<(f32, f32)> {
        self.progress.active(now, policy).then(|| {
            let p = self.progress.value(now, policy);
            (self.radius * p, 0.12 * (1.0 - p))
        })
    }
    pub fn request_frame<M>(
        &self,
        now: Instant,
        policy: Policy,
        shell: &mut iced_core::Shell<'_, M>,
    ) {
        self.progress.request_frame(now, policy, shell);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ON: Policy = Policy {
        system_enabled: true,
    };
    const OFF: Policy = Policy {
        system_enabled: false,
    };
    #[test]
    fn reversal_is_continuous_and_completion_stops_redraws() {
        let start = Instant::now();
        let middle = start + QUICK / 2;
        let mut tween = Tween::new(0.0, start);
        tween.retarget(1.0, start, QUICK, Easing::Linear, ON);
        assert_eq!(tween.value(middle, ON), 0.5);
        tween.retarget(0.0, middle, QUICK, Easing::Linear, ON);
        assert_eq!(tween.value(middle, ON), 0.5);
        let mut messages: Vec<()> = vec![];
        let mut shell = iced_core::Shell::new(&mut messages);
        tween.request_frame(middle + QUICK, ON, &mut shell);
        assert_eq!(
            shell.redraw_request(),
            iced_core::window::RedrawRequest::Wait
        );
        assert_eq!(tween.value(middle + QUICK, ON), 0.0);
        assert!(!tween.active(middle + QUICK, ON));
    }
    #[test]
    fn disabling_snaps_and_cannot_restart_old_motion() {
        let now = Instant::now();
        let mut tween = Tween::new(0.0, now);
        tween.retarget(1.0, now, STANDARD, Easing::EaseOut, ON);
        assert_eq!(tween.value(now, OFF), 1.0);
        tween.settle_if_disabled(OFF);
        assert_eq!(tween.value(now, ON), 1.0);
        assert!(!tween.active(now, ON));
        tween.retarget(0.0, now, Duration::ZERO, Easing::Linear, ON);
        assert_eq!(tween.value(now, ON), 0.0);
    }
    #[test]
    fn ripple_reaches_farthest_corner_and_disappears() {
        let now = Instant::now();
        let ripple = Ripple::new(
            iced_core::Point::ORIGIN,
            iced_core::Size::new(30.0, 40.0),
            now,
            ON,
        );
        assert_eq!(ripple.radius, 50.0);
        assert!(ripple.sample(now + QUICK, ON).is_some());
        assert!(ripple.sample(now + FEEDBACK, ON).is_none());
        assert!(ripple.sample(now, OFF).is_none());
    }

    #[test]
    fn frames_are_requested_only_for_enabled_active_transitions() {
        let now = Instant::now();
        let mut tween = Tween::new(0.0, now);
        tween.retarget(1.0, now, QUICK, Easing::EaseOut, ON);
        for (policy, expected) in [
            (ON, iced_core::window::RedrawRequest::NextFrame),
            (OFF, iced_core::window::RedrawRequest::Wait),
            (Policy::DISABLED, iced_core::window::RedrawRequest::Wait),
        ] {
            let mut messages: Vec<()> = vec![];
            let mut shell = iced_core::Shell::new(&mut messages);
            tween.request_frame(now, policy, &mut shell);
            assert_eq!(shell.redraw_request(), expected);
        }
    }
}
