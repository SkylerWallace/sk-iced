//! Motion policy only: no Iced renderer, executor, timers or event loop.
#[derive(Debug, Clone, Copy)]
pub enum QueryFallback {
    Disabled,
    Enabled,
}
impl QueryFallback {
    fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub system_enabled: bool,
}
impl Policy {
    pub const DISABLED: Self = Self {
        system_enabled: false,
    };
    pub fn enabled(self) -> bool {
        self.system_enabled
    }
    pub fn detect() -> Self {
        Self {
            system_enabled: system_enabled(QueryFallback::Disabled),
        }
    }
    pub fn refresh(&mut self) {
        self.system_enabled = system_enabled(QueryFallback::Disabled);
    }
}

pub fn system_enabled(fallback: QueryFallback) -> bool {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        let mut enabled = 0i32;
        // Failure conservatively disables decorative motion.
        if SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut enabled as *mut i32).cast(),
            0,
        ) != 0
        {
            enabled != 0
        } else {
            fallback.enabled()
        }
    }
    #[cfg(not(windows))]
    {
        fallback.enabled()
    }
}
