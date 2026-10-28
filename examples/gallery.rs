// SPDX-License-Identifier: Apache-2.0
//! Interactive gallery using unmodified, software-rendered Iced.
use iced::{
    Element, Fill, Task,
    widget::{column, container, row, text},
};
use sk_iced::{components::*, motion::Policy, palette, typography::TextRole};
fn main() -> iced::Result {
    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .title("sk-iced gallery")
        .theme(Gallery::theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1040., 840.),
            min_size: Some(iced::Size::new(540., 480.)),
            ..Default::default()
        })
        .subscription(|_| {
            iced::event::listen_with(|event, _, _| {
                matches!(event, iced::Event::Window(iced::window::Event::Focused))
                    .then_some(Message::RefreshPolicy)
            })
        })
        .run()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Default,
    Neumorphic,
    Material,
    Metro,
}
impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone)]
enum Message {
    Style(Style),
    Dark(bool),
    Disabled(bool),
    Reduced(bool),
    RefreshPolicy,
    Page(usize),
    Click,
    Text(String),
    Checked(bool),
    Switch(bool),
    Segment(usize),
    Choice(u8),
    Progress(f32),
    Expanded,
    Popup(bool),
    Tab(usize),
}
struct Gallery {
    style: Style,
    dark: bool,
    disabled: bool,
    reduced: bool,
    system: Policy,
    page: usize,
    clicks: u32,
    value: String,
    checked: bool,
    switch: bool,
    segment: usize,
    choice: u8,
    progress: f32,
    expanded: bool,
    popup: bool,
    tab: usize,
}
impl Default for Gallery {
    fn default() -> Self {
        Self {
            style: Style::Default,
            dark: false,
            disabled: false,
            reduced: false,
            system: Policy::detect(),
            page: 0,
            clicks: 0,
            value: "Editable text".into(),
            checked: true,
            switch: true,
            segment: 0,
            choice: 1,
            progress: 45.,
            expanded: false,
            popup: false,
            tab: 0,
        }
    }
}
impl Gallery {
    fn tokens(&self) -> sk_iced::theme::Tokens {
        let base = palette::palette(self.dark, false);
        match self.style {
            Style::Default => base,
            Style::Neumorphic => sk_iced::themes::neumorphic(base).tokens,
            Style::Material => sk_iced::themes::material(base).tokens,
            Style::Metro => sk_iced::themes::metro(base).tokens,
        }
    }
    fn theme(&self) -> iced::Theme {
        self.tokens().theme()
    }
    fn policy(&self) -> Policy {
        Policy {
            system_enabled: self.system.system_enabled && !self.reduced,
        }
    }
    fn ui(&self) -> UiContext {
        UiContext::new(
            Appearance::fixed(iced::Font::DEFAULT, self.tokens()),
            self.policy(),
        )
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Style(v) => self.style = v,
            Message::Dark(v) => self.dark = v,
            Message::Disabled(v) => self.disabled = v,
            Message::Reduced(v) => self.reduced = v,
            Message::RefreshPolicy => self.system.refresh(),
            Message::Page(v) => self.page = v,
            Message::Click => self.clicks += 1,
            Message::Text(v) => self.value = v,
            Message::Checked(v) => self.checked = v,
            Message::Switch(v) => self.switch = v,
            Message::Segment(v) => self.segment = v,
            Message::Choice(v) => self.choice = v,
            Message::Progress(v) => self.progress = v,
            Message::Expanded => self.expanded = !self.expanded,
            Message::Tab(v) => self.tab = v,
            Message::Popup(open) => {
                self.popup = open;
                if !open {
                    return iced::widget::operation::focus("popup-trigger");
                }
            }
        }
        Task::none()
    }
    fn controls(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let t = self.tokens();
        let enabled = !self.disabled;
        column![
            ui.text(TextRole::Heading, "Buttons and inputs"),
            row![
                ui.button("Default")
                    .on_press(Message::Click)
                    .enabled(enabled),
                ui.button("Primary")
                    .tone(ButtonTone::Important)
                    .on_press(Message::Click)
                    .enabled(enabled),
                ui.button("Danger")
                    .tone(ButtonTone::Danger)
                    .on_press(Message::Click)
                    .enabled(enabled),
                ui.button("Borderless")
                    .borderless()
                    .on_press(Message::Click)
                    .enabled(enabled),
            ]
            .spacing(10)
            .wrap(),
            ui.text_input("Placeholder", &self.value)
                .on_input(Message::Text)
                .enabled(enabled),
            row![
                ui.checkbox(self.checked)
                    .label("Checkbox")
                    .on_toggle(Message::Checked)
                    .enabled(enabled),
                ui.toggle_switch(self.switch)
                    .on_toggle(Message::Switch)
                    .enabled(enabled),
            ]
            .spacing(24),
            ui.segmented()
                .options([
                    Segment::new("First"),
                    Segment::new("Unavailable").enabled(false),
                    Segment::new("Third").tooltip("A choice with a tooltip"),
                ])
                .selected(self.segment)
                .on_select(Message::Segment)
                .enabled(enabled)
                .width(360),
            Dropdown::new(
                ui.appearance,
                vec![1u8, 2, 3],
                Some(self.choice),
                Message::Choice
            )
            .enabled(enabled)
            .width(180),
            Slider::new(0.0..=100.0, self.progress, Message::Progress)
                .with_appearance(ui.appearance)
                .enabled(enabled),
            LabeledProgress::new(
                ui.appearance,
                self.progress / 100.,
                t.accent,
                format!("{:.0}%", self.progress)
            ),
            row![
                StatusBadge::new(ui.appearance, "Ready", t.success),
                StatusBadge::new(ui.appearance, "Warning", t.warning),
                StatusBadge::new(ui.appearance, "Failed", t.failure),
            ]
            .spacing(12),
        ]
        .spacing(22)
        .into()
    }
    fn settings_cards(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let enabled = !self.disabled;
        let name = FormField::new(
            ui.appearance,
            "Project name",
            ui.text_input("Name", &self.value)
                .on_input(Message::Text)
                .enabled(enabled),
        )
        .hint("Validation belongs to the app.");
        let name = if self.value.trim().is_empty() {
            name.error("Enter a name.")
        } else {
            name
        };
        column![
            ui.text(TextRole::Heading, "Responsive settings cards"),
            ui.settings_card(
                "Notifications",
                ui.toggle_switch(self.switch)
                    .on_toggle(Message::Switch)
                    .enabled(enabled),
            )
            .description("A controlled value with a brief paint-only thumb transition."),
            ui.settings_card(
                "Display preference",
                Dropdown::new(
                    ui.appearance,
                    vec![1u8, 2, 3],
                    Some(self.choice),
                    Message::Choice
                )
                .width(160)
                .enabled(enabled),
            )
            .description(
                "Resize the window to see the label and action stack without losing state."
            ),
            ui.settings_card(
                "Name",
                ui.text_input("Name", &self.value)
                    .on_input(Message::Text)
                    .enabled(enabled)
                    .width(240),
            )
            .description("Input focus and selection survive card layout changes."),
            name,
        ]
        .spacing(12)
        .into()
    }
    fn workspace(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let enabled = !self.disabled;
        column![
            ui.text(TextRole::Heading, "Workspace composition"),
            row![
                Tab::new(
                    ui.appearance,
                    ui.motion,
                    "Overview",
                    self.tab == 0,
                    Some(Message::Tab(0))
                ),
                Tab::new(
                    ui.appearance,
                    ui.motion,
                    "Details",
                    self.tab == 1,
                    Some(Message::Tab(1))
                ),
            ]
            .spacing(4),
            Panel::new(
                ui.appearance,
                column![
                    text(if self.tab == 0 {
                        "Overview panel"
                    } else {
                        "Details panel"
                    })
                    .size(18),
                    ui.text_input("Shared state", &self.value)
                        .on_input(Message::Text)
                        .enabled(enabled),
                    ui.button("Panel action")
                        .on_press(Message::Click)
                        .enabled(enabled),
                ]
                .spacing(14),
            )
            .padding(20),
            DataTable::new(
                row![text("Name").width(Fill), text("Status").width(120)],
                column![
                    row![text("Component").width(Fill), text("Available").width(120)],
                    row![
                        text("Application state").width(Fill),
                        text("App-owned").width(120)
                    ],
                ]
                .spacing(12),
            ),
            TreeRow::new(
                ui.button(if self.expanded { "−" } else { "+" })
                    .on_press(Message::Expanded)
                    .enabled(enabled)
                    .width(28),
                text("Example branch"),
            )
            .depth(0),
            ExpandableCard::reveal(self.expanded, ui.motion, || {
                TreeRow::new(text(""), text("Child item"))
                    .depth(1)
                    .build()
                    .into()
            }),
        ]
        .spacing(20)
        .into()
    }
    fn motion_samples(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let enabled = !self.disabled;
        column![
            ui.text(TextRole::Heading, "Immediate actions, interruptible motion"),
            text("Click repeatedly or toggle reduced motion mid-transition. No global frame timer runs."),
            ui.button(if self.expanded { "Collapse card" } else { "Expand card" })
                .on_press(Message::Expanded).enabled(enabled),
            ExpandableCard::new(ui.appearance, ui.motion, text("Expandable card").size(16))
                .expanded(self.expanded)
                .body(move || {
                    column![
                        text("The body becomes actionable as soon as it opens."),
                        ui.button("Immediate action").on_press(Message::Click).enabled(enabled),
                    ].spacing(12).into()
                }),
            ui.button("Open popup").id("popup-trigger").on_press(Message::Popup(true)).enabled(enabled),
            ui.button("Ripple feedback").on_press(Message::Click).enabled(enabled),
        ].spacing(20).into()
    }
    fn view(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let settings = column![
            row![
                ui.text(TextRole::Title, "sk-iced"),
                text("Interactive component gallery").size(14),
            ].spacing(16).align_y(iced::Center),
            row![
                Dropdown::new(
                    ui.appearance,
                    vec![Style::Default, Style::Neumorphic, Style::Material, Style::Metro],
                    Some(self.style),
                    Message::Style,
                ).width(160),
                ui.checkbox(self.dark).label("Dark").on_toggle(Message::Dark),
                ui.checkbox(self.disabled).label("Disable samples").on_toggle(Message::Disabled),
            ].spacing(16).align_y(iced::Center).wrap(),
            row![
                ui.checkbox(self.reduced).label("Reduce motion").on_toggle(Message::Reduced),
                ui.button("Refresh system policy").on_press(Message::RefreshPolicy),
            ].spacing(16).align_y(iced::Center).wrap(),
            text(format!(
                "Motion: {} · Commands: {} · Tab / Shift+Tab to navigate; Space / Enter to activate",
                if self.policy().enabled() { "enabled" } else { "disabled" },
                self.clicks,
            )).size(12),
            ui.segmented()
                .labels(["Controls", "Settings & cards", "Workspace", "Motion & overlays"])
                .selected(self.page).on_select(Message::Page),
        ].spacing(12);
        let samples = match self.page {
            0 => self.controls(),
            1 => self.settings_cards(),
            2 => self.workspace(),
            _ => self.motion_samples(),
        };
        let base = container(
            column![
                settings,
                ScrollArea::new(ui.appearance, container(samples).padding(16)).height(Fill),
            ]
            .spacing(20),
        )
        .padding(24)
        .width(Fill)
        .height(Fill);
        let base = FocusScope::new(base).active(!self.popup);
        let popup = FocusScope::new(
            SettingsCard::new(
                ui.appearance,
                "Popup focus",
                column![
                    text("Escape or Close dismisses and restores focus to the trigger."),
                    ui.button("Close")
                        .id("popup-close")
                        .on_press(Message::Popup(false)),
                ]
                .spacing(16),
            )
            .stacked(true),
        )
        .active(self.popup)
        .autofocus("popup-close");
        let popup = Popup::new(base, container(popup).width(440))
            .open(self.popup)
            .policy(ui.motion)
            .centered(true)
            .on_dismiss(Message::Popup(false));
        RedrawBoundary::new(popup).full_frame(true).into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn gallery_views_fit_the_native_windows_stack() {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                for page in 0..4 {
                    let app = super::Gallery {
                        page,
                        ..super::Gallery::default()
                    };
                    let _view = app.view();
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
    use super::*;
    use iced_core::{
        Layout, Renderer as _, layout, mouse, renderer, renderer::Headless as _, widget::Tree,
    };

    #[test]
    fn every_gallery_page_renders_with_stock_iced_in_each_style_and_mode() {
        for style in [
            Style::Default,
            Style::Neumorphic,
            Style::Material,
            Style::Metro,
        ] {
            for dark in [false, true] {
                for page in 0..4 {
                    let app = Gallery {
                        style,
                        dark,
                        page,
                        reduced: true,
                        ..Default::default()
                    };
                    let mut view = app.view();
                    let mut tree = Tree::new(&view);
                    let mut renderer = sk_iced::Renderer::new(iced::Font::DEFAULT, 13.333.into());
                    let size = iced::Size::new(960., 800.);
                    let bounds = iced::Rectangle::with_size(size);
                    let node = view.as_widget_mut().layout(
                        &mut tree,
                        &renderer,
                        &layout::Limits::new(size, size),
                    );
                    assert_eq!(node.size(), size);
                    let mut messages = Vec::new();
                    view.as_widget_mut().update(
                        &mut tree,
                        &iced::Event::Window(iced::window::Event::RedrawRequested(
                            std::time::Instant::now(),
                        )),
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &renderer,
                        &mut iced_core::clipboard::Null,
                        &mut iced_core::Shell::new(&mut messages),
                        &bounds,
                    );
                    renderer.reset(bounds);
                    view.as_widget().draw(
                        &tree,
                        &mut renderer,
                        &app.theme(),
                        &renderer::Style {
                            text_color: app.theme().palette().text,
                        },
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &bounds,
                    );
                    let pixels = renderer.screenshot(
                        iced::Size::new(960, 800),
                        1.,
                        app.theme().palette().background,
                    );
                    assert_eq!(pixels.len(), 960 * 800 * 4);
                    if std::env::var_os("SK_ICED_CAPTURE").is_some() && page == 0 {
                        use std::io::Write;
                        let output = std::env::var_os("SK_ICED_CAPTURE").unwrap();
                        std::fs::create_dir_all(&output).unwrap();
                        let path =
                            std::path::PathBuf::from(output).join(format!("{style:?}-{dark}.ppm"));
                        let mut file =
                            std::io::BufWriter::new(std::fs::File::create(path).unwrap());
                        write!(file, "P6\n960 800\n255\n").unwrap();
                        for pixel in pixels.as_chunks::<4>().0 {
                            file.write_all(&pixel[..3]).unwrap();
                        }
                    }
                }
            }
        }
    }
}
