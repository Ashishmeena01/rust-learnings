//! A modern, beautiful, and interactive Terminal Calendar built with Ratatui.
//!
//! Designed as an elegant Browser Page with a Black & White (Monochrome) theme
//! and authentic card drop shadows for 3D elevation.
//!
//! Features:
//! - Browser Chrome: window controls (● ● ●), active tabs, dynamic Omnibox URL bar, and status bar.
//! - Monochrome Themes: Obsidian Dark, Paper White, Neobrutalist B&W, and Tokyo Night (Color).
//! - 3D Card Shadows: Real terminal drop shadows cast by every card.
//! - Responsive Views: 12-Month Year Overview and Focused Month Zoom.
//! - Live Date Inspector: Season tracker, Year progress bar, and Holiday countdown.
//! - Comprehensive Keybindings: Day (hjkl/arrows), Month (n/p/Tab), Year (y/Y/[]), Jump to Today (t).

use std::fmt;

use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use time::ext::NumericalDuration;
use time::{Date, Month, OffsetDateTime};

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(run)
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let today = OffsetDateTime::now_local()
        .map(|dt| dt.date())
        .unwrap_or_else(|_| Date::from_calendar_date(2026, Month::January, 1).unwrap());

    let mut selected_date = today;
    let mut calendar_style = StyledCalendar::MonthAndWeekdaysHeader;
    let mut theme = ThemeKind::MonochromeDark;
    let mut view_mode = ViewMode::Year;
    let mut show_help = false;

    loop {
        terminal.draw(|frame| {
            render(
                frame,
                calendar_style,
                selected_date,
                today,
                theme,
                view_mode,
                show_help,
            )
        })?;

        if let Ok(event) = event::read() {
            if let Some(key) = event.as_key_press_event() {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if show_help {
                    match key.code {
                        KeyCode::Char('q') => break Ok(()),
                        _ => show_help = false,
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('?') => show_help = true,
                    KeyCode::Char('c') => theme = theme.next(),
                    KeyCode::Char('v') | KeyCode::Enter => view_mode = view_mode.toggle(),
                    KeyCode::Char('s') => calendar_style = calendar_style.next(),
                    KeyCode::Char('t') => selected_date = today,
                    KeyCode::Char('y') | KeyCode::Char(']') => {
                        selected_date = change_year(selected_date, 1);
                    }
                    KeyCode::Char('Y') | KeyCode::Char('[') => {
                        selected_date = change_year(selected_date, -1);
                    }
                    KeyCode::Char('n') | KeyCode::Tab => {
                        selected_date = change_month(selected_date, 1);
                    }
                    KeyCode::Char('p') | KeyCode::BackTab => {
                        selected_date = change_month(selected_date, -1);
                    }
                    KeyCode::Char('h') | KeyCode::Left => selected_date -= 1.days(),
                    KeyCode::Char('l') | KeyCode::Right => selected_date += 1.days(),
                    KeyCode::Char('j') | KeyCode::Down => selected_date += 1.weeks(),
                    KeyCode::Char('k') | KeyCode::Up => selected_date -= 1.weeks(),
                    _ => {}
                }
            }
        }
    }
}

// ==========================================
// Safe Date Navigation Helpers
// ==========================================

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: Month) -> u8 {
    match month {
        Month::January => 31,
        Month::February => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        Month::March => 31,
        Month::April => 30,
        Month::May => 31,
        Month::June => 30,
        Month::July => 31,
        Month::August => 31,
        Month::September => 30,
        Month::October => 31,
        Month::November => 30,
        Month::December => 31,
    }
}

fn days_in_year(year: i32) -> u16 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn change_year(date: Date, delta: i32) -> Date {
    let target_year = date.year() + delta;
    let max_day = days_in_month(target_year, date.month());
    let clamped_day = date.day().min(max_day);
    Date::from_calendar_date(target_year, date.month(), clamped_day).unwrap()
}

fn change_month(date: Date, delta: i32) -> Date {
    let year = date.year();
    let current_m = date.month() as i32;
    let new_m = current_m + delta;

    let (target_year, target_month_num) = if new_m > 12 {
        let yr_add = (new_m - 1) / 12;
        let m = ((new_m - 1) % 12) + 1;
        (year + yr_add, m as u8)
    } else if new_m < 1 {
        let yr_sub = (12 - new_m) / 12;
        let m = 12 - ((12 - new_m) % 12);
        let m = if m == 0 { 12 } else { m };
        (year - yr_sub, m as u8)
    } else {
        (year, new_m as u8)
    };

    let target_month = Month::try_from(target_month_num).unwrap();
    let max_day = days_in_month(target_year, target_month);
    let clamped_day = date.day().min(max_day);
    Date::from_calendar_date(target_year, target_month, clamped_day).unwrap()
}

fn get_season(date: Date) -> (&'static str, &'static str) {
    let (m, d) = (date.month(), date.day());
    match m {
        Month::March if d >= 20 => ("Spring", "🌸"),
        Month::April | Month::May => ("Spring", "🌸"),
        Month::June if d < 21 => ("Spring", "🌸"),

        Month::June if d >= 21 => ("Summer", "☀️"),
        Month::July | Month::August => ("Summer", "☀️"),
        Month::September if d < 22 => ("Summer", "☀️"),

        Month::September if d >= 22 => ("Autumn", "🍂"),
        Month::October | Month::November => ("Autumn", "🍂"),
        Month::December if d < 21 => ("Autumn", "🍂"),

        _ => ("Winter", "❄️"),
    }
}

// ==========================================
// Events & Observances
// ==========================================

#[derive(Debug, Clone, Copy)]
pub struct CalendarEvent {
    pub month: Month,
    pub day: u8,
    pub name: &'static str,
    pub icon: &'static str,
    pub is_holiday: bool,
}

pub const CALENDAR_EVENTS: &[CalendarEvent] = &[
    CalendarEvent {
        month: Month::January,
        day: 1,
        name: "New Year's Day",
        icon: "🎉",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::January,
        day: 26,
        name: "Republic Day",
        icon: "🇮🇳",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::February,
        day: 2,
        name: "Groundhog Day",
        icon: "🦫",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::February,
        day: 14,
        name: "Valentine's Day",
        icon: "💖",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::March,
        day: 8,
        name: "Int'l Women's Day",
        icon: "🌸",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::March,
        day: 20,
        name: "Spring Equinox",
        icon: "🌱",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::April,
        day: 1,
        name: "April Fools' Day",
        icon: "🃏",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::April,
        day: 22,
        name: "Earth Day",
        icon: "🌍",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::May,
        day: 1,
        name: "Labor Day",
        icon: "🛠️",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::May,
        day: 4,
        name: "Star Wars Day",
        icon: "⭐",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::June,
        day: 21,
        name: "Summer Solstice",
        icon: "☀️",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::July,
        day: 4,
        name: "Independence Day",
        icon: "🎆",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::August,
        day: 15,
        name: "Independence Day (IN)",
        icon: "🇮🇳",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::September,
        day: 22,
        name: "Autumn Equinox",
        icon: "🍂",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::October,
        day: 31,
        name: "Halloween",
        icon: "🎃",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::November,
        day: 11,
        name: "Veterans Day",
        icon: "🎖️",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::December,
        day: 21,
        name: "Winter Solstice",
        icon: "❄️",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::December,
        day: 23,
        name: "Festivus",
        icon: "🎄",
        is_holiday: false,
    },
    CalendarEvent {
        month: Month::December,
        day: 24,
        name: "Christmas Eve",
        icon: "✨",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::December,
        day: 25,
        name: "Christmas Day",
        icon: "🎁",
        is_holiday: true,
    },
    CalendarEvent {
        month: Month::December,
        day: 31,
        name: "New Year's Eve",
        icon: "🥂",
        is_holiday: true,
    },
];

fn find_event(date: Date) -> Option<&'static CalendarEvent> {
    CALENDAR_EVENTS
        .iter()
        .find(|e| e.month == date.month() && e.day == date.day())
}

fn events_in_month(month: Month) -> Vec<&'static CalendarEvent> {
    CALENDAR_EVENTS.iter().filter(|e| e.month == month).collect()
}

fn next_event(from_date: Date) -> Option<(&'static CalendarEvent, i64)> {
    let mut min_diff = i64::MAX;
    let mut closest = None;

    for event in CALENDAR_EVENTS {
        for year_offset in [0, 1] {
            if let Ok(event_date) =
                Date::from_calendar_date(from_date.year() + year_offset, event.month, event.day)
            {
                let diff = (event_date - from_date).whole_days();
                if diff >= 0 && diff < min_diff {
                    min_diff = diff;
                    closest = Some((event, diff));
                }
            }
        }
    }
    closest
}

// ==========================================
// Theme Definitions (Black & White Primary)
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    MonochromeDark,  // Obsidian Dark with crisp silver white and soft dither shadows
    MonochromeLight, // Paper White canvas with dark ink cards and cast shadows
    Neobrutalist,    // High contrast black & white with solid 3D shadows
    TokyoNight,      // Cyberpunk Color Alternative
}

impl ThemeKind {
    pub const fn next(self) -> Self {
        match self {
            Self::MonochromeDark => Self::MonochromeLight,
            Self::MonochromeLight => Self::Neobrutalist,
            Self::Neobrutalist => Self::TokyoNight,
            Self::TokyoNight => Self::MonochromeDark,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::MonochromeDark => "B&W Obsidian",
            Self::MonochromeLight => "B&W Paper",
            Self::Neobrutalist => "Neobrutalist B&W",
            Self::TokyoNight => "Tokyo Night",
        }
    }

    pub fn canvas_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Black,
            Self::MonochromeLight => Color::Rgb(242, 242, 246),
            Self::Neobrutalist => Color::Black,
            Self::TokyoNight => Color::Rgb(18, 19, 28),
        }
    }

    pub fn card_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(16, 16, 16),
            Self::MonochromeLight => Color::White,
            Self::Neobrutalist => Color::Rgb(10, 10, 10),
            Self::TokyoNight => Color::Rgb(26, 27, 38),
        }
    }

    pub fn chrome_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(24, 24, 24),
            Self::MonochromeLight => Color::Rgb(228, 228, 233),
            Self::Neobrutalist => Color::Rgb(20, 20, 20),
            Self::TokyoNight => Color::Rgb(32, 34, 48),
        }
    }

    pub fn omnibox_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(10, 10, 10),
            Self::MonochromeLight => Color::White,
            Self::Neobrutalist => Color::Black,
            Self::TokyoNight => Color::Rgb(20, 21, 32),
        }
    }

    pub fn border(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(75, 75, 75),
            Self::MonochromeLight => Color::Rgb(180, 180, 185),
            Self::Neobrutalist => Color::Rgb(140, 140, 140),
            Self::TokyoNight => Color::Rgb(65, 72, 104),
        }
    }

    pub fn border_active(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(125, 207, 255),
        }
    }

    pub fn shadow_fg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(60, 60, 60),
            Self::MonochromeLight => Color::Rgb(190, 190, 195),
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(45, 48, 68),
        }
    }

    pub fn shadow_char_side(self) -> &'static str {
        match self {
            Self::Neobrutalist => "▌",
            _ => "░",
        }
    }

    pub fn shadow_char_bot(self) -> &'static str {
        match self {
            Self::Neobrutalist => "▄",
            _ => "░",
        }
    }

    pub fn shadow_char_corner(self) -> &'static str {
        match self {
            Self::Neobrutalist => "▟",
            _ => "░",
        }
    }

    pub fn text(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::White,
        }
    }

    pub fn dim(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(110, 110, 110),
            Self::MonochromeLight => Color::Rgb(140, 140, 140),
            Self::Neobrutalist => Color::Rgb(110, 110, 110),
            Self::TokyoNight => Color::Rgb(86, 95, 137),
        }
    }

    pub fn selected_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(247, 118, 142),
        }
    }

    pub fn selected_fg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Black,
            Self::MonochromeLight => Color::White,
            Self::Neobrutalist => Color::Black,
            Self::TokyoNight => Color::Black,
        }
    }

    pub fn today_bg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(50, 50, 50),
            Self::MonochromeLight => Color::Rgb(215, 215, 220),
            Self::Neobrutalist => Color::Rgb(55, 55, 55),
            Self::TokyoNight => Color::Rgb(115, 218, 202),
        }
    }

    pub fn today_fg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Black,
        }
    }

    pub fn holiday_fg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(255, 158, 100),
        }
    }

    pub fn season_fg(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(190, 190, 190),
            Self::MonochromeLight => Color::Rgb(50, 50, 50),
            Self::Neobrutalist => Color::Rgb(190, 190, 190),
            Self::TokyoNight => Color::Rgb(224, 175, 104),
        }
    }

    pub fn primary(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(122, 162, 247),
        }
    }

    pub fn secondary(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(200, 200, 200),
            Self::MonochromeLight => Color::Rgb(55, 55, 55),
            Self::Neobrutalist => Color::Rgb(200, 200, 200),
            Self::TokyoNight => Color::Rgb(187, 154, 247),
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::MonochromeDark => Color::White,
            Self::MonochromeLight => Color::Black,
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(125, 207, 255),
        }
    }

    pub fn dot_red(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(225, 225, 225),
            Self::MonochromeLight => Color::Rgb(50, 50, 50),
            Self::Neobrutalist => Color::White,
            Self::TokyoNight => Color::Rgb(247, 118, 142),
        }
    }

    pub fn dot_yellow(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(165, 165, 165),
            Self::MonochromeLight => Color::Rgb(115, 115, 115),
            Self::Neobrutalist => Color::Rgb(160, 160, 160),
            Self::TokyoNight => Color::Rgb(224, 175, 104),
        }
    }

    pub fn dot_green(self) -> Color {
        match self {
            Self::MonochromeDark => Color::Rgb(115, 115, 115),
            Self::MonochromeLight => Color::Rgb(175, 175, 175),
            Self::Neobrutalist => Color::Rgb(100, 100, 100),
            Self::TokyoNight => Color::Rgb(115, 218, 202),
        }
    }
}

// ==========================================
// View Mode & Calendar Styles
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Year,
    Month,
}

impl ViewMode {
    pub const fn toggle(self) -> Self {
        match self {
            Self::Year => Self::Month,
            Self::Month => Self::Year,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Year => "Year Overview",
            Self::Month => "Focused Month",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StyledCalendar {
    Default,
    Surrounding,
    WeekdaysHeader,
    SurroundingAndWeekdaysHeader,
    MonthHeader,
    MonthAndWeekdaysHeader,
}

impl StyledCalendar {
    const fn next(self) -> Self {
        match self {
            Self::MonthAndWeekdaysHeader => Self::SurroundingAndWeekdaysHeader,
            Self::SurroundingAndWeekdaysHeader => Self::WeekdaysHeader,
            Self::WeekdaysHeader => Self::MonthHeader,
            Self::MonthHeader => Self::Surrounding,
            Self::Surrounding => Self::Default,
            Self::Default => Self::MonthAndWeekdaysHeader,
        }
    }
}

impl fmt::Display for StyledCalendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "Minimal"),
            Self::Surrounding => write!(f, "Surrounding Days"),
            Self::WeekdaysHeader => write!(f, "Weekdays Only"),
            Self::SurroundingAndWeekdaysHeader => write!(f, "Surrounding + Weekdays"),
            Self::MonthHeader => write!(f, "Month Title"),
            Self::MonthAndWeekdaysHeader => write!(f, "Full Headers"),
        }
    }
}

// ==========================================
// 3D Card Shadow Rendering Engine
// ==========================================

fn render_card_shadow(frame: &mut Frame, area: Rect, theme: ThemeKind) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    let buf = frame.buffer_mut();
    let shadow_style = Style::default()
        .fg(theme.shadow_fg())
        .bg(theme.canvas_bg());

    let right_x = area.x + area.width;
    let bot_y = area.y + area.height;

    // Right shadow: 1 column to the right, from y+1 down to bot_y-1
    if right_x < buf.area.width {
        for y in (area.y + 1)..bot_y {
            if y < buf.area.height {
                buf.set_string(right_x, y, theme.shadow_char_side(), shadow_style);
            }
        }
    }

    // Bottom shadow: 1 row below, from x+1 across to right_x-1
    if bot_y < buf.area.height {
        for x in (area.x + 1)..right_x {
            if x < buf.area.width {
                buf.set_string(x, bot_y, theme.shadow_char_bot(), shadow_style);
            }
        }
    }

    // Bottom-right corner shadow
    if right_x < buf.area.width && bot_y < buf.area.height {
        buf.set_string(right_x, bot_y, theme.shadow_char_corner(), shadow_style);
    }
}

impl StyledCalendar {
    fn render_month(
        self,
        frame: &mut Frame,
        area: Rect,
        date: Date,
        events: &CalendarEventStore,
        theme: ThemeKind,
        is_selected_month: bool,
    ) {
        let def_style = Style::new().bold().bg(theme.card_bg()).fg(theme.text());
        let month_hdr_style = if is_selected_month {
            Style::new().bold().fg(theme.accent())
        } else {
            Style::new().bold().fg(theme.secondary())
        };
        let weekday_style = Style::new().bold().fg(theme.primary());
        let surrounding_style = Style::new().fg(theme.dim());

        // When area is spacious enough, wrap month in a rounded card with 3D drop shadow
        if area.width >= 24 && area.height >= 9 {
            let card_rect = Rect {
                x: area.x,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height.saturating_sub(1),
            };

            // Cast drop shadow onto the canvas
            render_card_shadow(frame, card_rect, theme);

            let month_name = date.month().to_string();
            let border_color = if is_selected_month {
                theme.border_active()
            } else {
                theme.border()
            };
            let title = if is_selected_month {
                format!(" ✦ {month_name} ✦ ")
            } else {
                format!(" {month_name} ")
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if is_selected_month {
                    Style::default().fg(border_color).bold()
                } else {
                    Style::default().fg(border_color)
                })
                .title(
                    Line::from(title).style(if is_selected_month {
                        Style::default().fg(theme.accent()).bold()
                    } else {
                        Style::default().fg(theme.primary())
                    }),
                )
                .style(Style::default().bg(theme.card_bg()).fg(theme.text()));

            let inner = block.inner(card_rect);
            frame.render_widget(block, card_rect);

            let calendar = match self {
                Self::Surrounding => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_surrounding(surrounding_style),
                Self::WeekdaysHeader | Self::Default => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_weekdays_header(weekday_style),
                Self::SurroundingAndWeekdaysHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_surrounding(surrounding_style)
                    .show_weekdays_header(weekday_style),
                Self::MonthHeader => Monthly::new(date, events).default_style(def_style),
                Self::MonthAndWeekdaysHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_weekdays_header(weekday_style),
            };
            frame.render_widget(calendar, inner);
        } else {
            let calendar = match self {
                Self::Default => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style),
                Self::Surrounding => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style)
                    .show_surrounding(surrounding_style),
                Self::WeekdaysHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style)
                    .show_weekdays_header(weekday_style),
                Self::SurroundingAndWeekdaysHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style)
                    .show_surrounding(surrounding_style)
                    .show_weekdays_header(weekday_style),
                Self::MonthHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style),
                Self::MonthAndWeekdaysHeader => Monthly::new(date, events)
                    .default_style(def_style)
                    .show_month_header(month_hdr_style)
                    .show_weekdays_header(weekday_style),
            };
            frame.render_widget(calendar, area);
        }
    }
}

// ==========================================
// Event Store Construction
// ==========================================

fn create_event_store(selected_date: Date, theme: ThemeKind) -> Result<CalendarEventStore> {
    let today_style = Style::default()
        .fg(theme.today_fg())
        .bg(theme.today_bg())
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let mut list = CalendarEventStore::today(today_style);
    let y = selected_date.year();

    let holiday_style = Style::new()
        .fg(theme.holiday_fg())
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let season_style = Style::new()
        .fg(theme.season_fg())
        .add_modifier(Modifier::BOLD);

    for event in CALENDAR_EVENTS {
        let style = if event.is_holiday {
            holiday_style
        } else {
            season_style
        };

        if let Ok(d) = Date::from_calendar_date(y, event.month, event.day) {
            list.add(d, style);
        }
        if event.month == Month::January && event.day == 1 {
            if let Ok(d) = Date::from_calendar_date(y + 1, event.month, event.day) {
                list.add(d, style);
            }
        }
    }

    // High contrast inverted selected date cell
    let selected_style = Style::new()
        .fg(theme.selected_fg())
        .bg(theme.selected_bg())
        .add_modifier(Modifier::BOLD);

    list.add(selected_date, selected_style);

    Ok(list)
}

// ==========================================
// UI Rendering Engine (Browser Page Layout)
// ==========================================

fn render(
    frame: &mut Frame,
    calendar_style: StyledCalendar,
    selected_date: Date,
    today: Date,
    theme: ThemeKind,
    view_mode: ViewMode,
    show_help: bool,
) {
    let screen = frame.area();

    // Fill screen background canvas
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.canvas_bg())),
        screen,
    );

    // Layout: Browser Chrome (4 rows) + Page Content (Flex 1) + Browser Status Bar (2 rows)
    let [header_area, content_area, footer_area] = screen.layout(&Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(2),
    ]));

    render_browser_header(frame, header_area, selected_date, today, theme, view_mode);
    render_browser_footer(frame, footer_area, theme, calendar_style);

    let events = create_event_store(selected_date, theme).unwrap();

    match view_mode {
        ViewMode::Year => {
            render_year_view(
                frame,
                content_area,
                calendar_style,
                selected_date,
                today,
                &events,
                theme,
            );
        }
        ViewMode::Month => {
            render_focused_month_view(
                frame,
                content_area,
                calendar_style,
                selected_date,
                today,
                &events,
                theme,
            );
        }
    }

    if show_help {
        render_help_modal(frame, theme);
    }
}

// ==========================================
// Browser Window Chrome & Omnibox Bar
// ==========================================

fn render_browser_header(
    frame: &mut Frame,
    area: Rect,
    selected_date: Date,
    today: Date,
    theme: ThemeKind,
    view_mode: ViewMode,
) {
    let (season_name, season_icon) = get_season(selected_date);
    let weekday_name = selected_date.weekday().to_string();
    let month_name = selected_date.month().to_string();
    let year = selected_date.year();
    let day = selected_date.day();
    let week_num = selected_date.iso_week();
    let day_of_year = selected_date.ordinal();
    let total_days = days_in_year(year);
    let is_today = selected_date == today;

    // Line 0: Browser Window Title & Active Tab
    let line0 = Line::from(vec![
        Span::styled(" ● ", Style::default().fg(theme.dot_red())),
        Span::styled("● ", Style::default().fg(theme.dot_yellow())),
        Span::styled("●  ", Style::default().fg(theme.dot_green())),
        Span::styled("│ ", Style::default().fg(theme.border())),
        Span::styled("╭── ", Style::default().fg(theme.border())),
        Span::styled(
            "🗓️  Chronos Web Calendar — ",
            Style::default().fg(theme.text()).bold(),
        ),
        Span::styled(format!("{year} "), Style::default().fg(theme.text()).bold()),
        Span::styled("──╮ ", Style::default().fg(theme.border())),
        Span::styled("＋  ", Style::default().fg(theme.dim())),
        Span::styled(
            format!("[ 🌐 browser://calendar/{year} ]"),
            Style::default().fg(theme.dim()),
        ),
    ]);

    // Line 1: Omnibox / Address Bar with dynamic URL reflecting active state
    let month_slug = month_name.to_lowercase();
    let view_slug = match view_mode {
        ViewMode::Year => "year",
        ViewMode::Month => "month",
    };
    let url_str = format!(
        " 🔒 https://chronos.local/{year}/{month_slug}?day={day:02}&view={view_slug} "
    );

    let line1 = Line::from(vec![
        Span::styled("  ◀  ▶  ⟳  │ ", Style::default().fg(theme.text())),
        Span::styled(
            format!("{url_str:<54}"),
            Style::default().fg(theme.text()).bg(theme.omnibox_bg()).bold(),
        ),
        Span::styled("   ★  ⎙  ⚙ ", Style::default().fg(theme.dim())),
    ]);

    // Line 2: Web App Navigation Bar & Selected Date Status
    let mut line2_spans = vec![
        Span::styled(
            "  CHRONOS.IO  ",
            Style::default()
                .fg(theme.selected_fg())
                .bg(theme.selected_bg())
                .bold(),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border())),
        Span::styled(
            format!("✨ {weekday_name}, {month_name} {day:02}, {year} "),
            Style::default().fg(theme.text()).bold(),
        ),
        Span::styled("• ", Style::default().fg(theme.dim())),
        Span::styled(
            format!("Week {week_num:02} "),
            Style::default().fg(theme.secondary()),
        ),
        Span::styled("• ", Style::default().fg(theme.dim())),
        Span::styled(
            format!("Day {day_of_year}/{total_days} "),
            Style::default().fg(theme.dim()),
        ),
        Span::styled("• ", Style::default().fg(theme.dim())),
        Span::styled(
            format!("{season_icon} {season_name} "),
            Style::default().fg(theme.season_fg()),
        ),
    ];

    if is_today {
        line2_spans.push(Span::styled("• ", Style::default().fg(theme.dim())));
        line2_spans.push(Span::styled(
            "● TODAY ",
            Style::default().fg(theme.accent()).bold().underlined(),
        ));
    }

    line2_spans.push(Span::styled(
        format!("  [🎨 {}] ", theme.name()),
        Style::default().fg(theme.secondary()),
    ));
    line2_spans.push(Span::styled(
        format!("[🔍 {}]", view_mode.name()),
        Style::default().fg(theme.accent()),
    ));

    let line2 = Line::from(line2_spans);

    // Line 3: Clean horizontal rule beneath browser chrome
    let line3 = Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(theme.border()),
    ));

    let p = Paragraph::new(vec![line0, line1, line2, line3])
        .style(Style::default().bg(theme.chrome_bg()));
    frame.render_widget(p, area);
}

// ==========================================
// Browser Status Bar Footer
// ==========================================

fn render_browser_footer(
    frame: &mut Frame,
    area: Rect,
    theme: ThemeKind,
    calendar_style: StyledCalendar,
) {
    let line0 = Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(theme.border()),
    ));

    let pill = |key: &'static str, label: &'static str| -> Vec<Span<'static>> {
        vec![
            Span::styled(
                format!(" {key} "),
                Style::default()
                    .fg(theme.selected_fg())
                    .bg(theme.selected_bg())
                    .bold(),
            ),
            Span::styled(format!(" {label}  "), Style::default().fg(theme.text())),
        ]
    };

    let style_desc = format!("{calendar_style}");
    let mut spans = vec![
        Span::styled(
            " 🔒 TLS 1.3  100% Zoom  │ ",
            Style::default().fg(theme.dim()),
        ),
    ];
    spans.extend(pill("q", "Quit"));
    spans.extend(pill("←↓↑→/hjkl", "Move"));
    spans.extend(pill("n/p", "Month"));
    spans.extend(pill("y/Y", "Year"));
    spans.extend(pill("t", "Today"));
    spans.extend(pill("c", "Theme"));
    spans.extend(pill("v", "View"));
    spans.push(Span::styled(
        " s ",
        Style::default()
            .fg(theme.selected_fg())
            .bg(theme.selected_bg())
            .bold(),
    ));
    spans.push(Span::styled(
        format!(" Style ({style_desc})  "),
        Style::default().fg(theme.text()),
    ));
    spans.extend(pill("?", "Help"));

    let line1 = Line::from(spans);

    let p = Paragraph::new(vec![line0, line1]).style(Style::default().bg(theme.chrome_bg()));
    frame.render_widget(p, area);
}

// ==========================================
// Year View with Card Elevation & Shadows
// ==========================================

fn render_year_view(
    frame: &mut Frame,
    area: Rect,
    calendar_style: StyledCalendar,
    selected_date: Date,
    today: Date,
    events: &CalendarEventStore,
    theme: ThemeKind,
) {
    if area.width >= 115 && area.height >= 26 {
        let [cal_area, side_area] = area.layout(&Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(34),
        ]));
        render_year_grid(
            frame,
            cal_area,
            calendar_style,
            selected_date,
            events,
            theme,
        );
        render_inspector(frame, side_area, selected_date, today, theme);
    } else if area.height >= 38 {
        let [cal_area, bot_area] = area.layout(&Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(8),
        ]));
        render_year_grid(
            frame,
            cal_area,
            calendar_style,
            selected_date,
            events,
            theme,
        );
        render_inspector(frame, bot_area, selected_date, today, theme);
    } else {
        render_year_grid(frame, area, calendar_style, selected_date, events, theme);
    }
}

fn render_year_grid(
    frame: &mut Frame,
    area: Rect,
    calendar_style: StyledCalendar,
    selected_date: Date,
    events: &CalendarEventStore,
    theme: ThemeKind,
) {
    let vertical = Layout::vertical([Constraint::Ratio(1, 3); 3]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, 4); 4]);

    let areas = area
        .inner(Margin::new(1, 0))
        .layout_vec(&vertical)
        .into_iter()
        .flat_map(|row| row.layout_vec(&horizontal));

    for (i, month_area) in areas.enumerate() {
        let target_month = Month::try_from(i as u8 + 1).unwrap();
        let month_date = selected_date
            .replace_day(1)
            .unwrap()
            .replace_month(target_month)
            .unwrap();

        let is_selected_month = target_month == selected_date.month();
        calendar_style.render_month(
            frame,
            month_area,
            month_date,
            events,
            theme,
            is_selected_month,
        );
    }
}

// ==========================================
// Focused Month View with Mini Previews
// ==========================================

fn render_focused_month_view(
    frame: &mut Frame,
    area: Rect,
    calendar_style: StyledCalendar,
    selected_date: Date,
    today: Date,
    events: &CalendarEventStore,
    theme: ThemeKind,
) {
    let [left_area, right_area] = area.layout(&Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ]));

    // Left: Spacious Month View Card with 3D drop shadow
    let month_name = selected_date.month().to_string();
    let year = selected_date.year();

    let left_card_rect = Rect {
        x: left_area.x,
        y: left_area.y,
        width: left_area.width.saturating_sub(1),
        height: left_area.height.saturating_sub(1),
    };
    render_card_shadow(frame, left_card_rect, theme);

    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active()).bold())
        .title(
            Line::from(format!(" ✦ {month_name} {year} ✦ "))
                .bold()
                .style(Style::default().fg(theme.accent())),
        )
        .style(Style::default().bg(theme.card_bg()).fg(theme.text()));

    let inner_left = left_block.inner(left_card_rect);
    frame.render_widget(left_block, left_card_rect);

    // Centered calendar inside left card
    let cal_centered = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(9),
        Constraint::Fill(1),
    ])
    .split(inner_left)[1];

    let cal_box = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(22),
        Constraint::Fill(1),
    ])
    .split(cal_centered)[1];

    calendar_style.render_month(
        frame,
        cal_box,
        selected_date,
        events,
        theme,
        true,
    );

    // Right: Date Inspector + Month Events + Mini Prev/Next Month Previews
    let [inspector_area, events_area, mini_area] = right_area.layout(&Layout::vertical([
        Constraint::Length(8),
        Constraint::Fill(1),
        Constraint::Length(10),
    ]));

    render_inspector(frame, inspector_area, selected_date, today, theme);
    render_month_events_card(frame, events_area, selected_date.month(), theme);

    // Mini preview for Prev and Next Month
    let [prev_area, next_area] = mini_area.layout(&Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 2),
    ]));

    let prev_month_date = change_month(selected_date, -1);
    let next_month_date = change_month(selected_date, 1);

    calendar_style.render_month(
        frame,
        prev_area,
        prev_month_date,
        events,
        theme,
        false,
    );
    calendar_style.render_month(
        frame,
        next_area,
        next_month_date,
        events,
        theme,
        false,
    );
}

// ==========================================
// Inspector Card with 3D Shadow
// ==========================================

fn render_inspector(
    frame: &mut Frame,
    area: Rect,
    selected_date: Date,
    today: Date,
    theme: ThemeKind,
) {
    if area.width < 5 || area.height < 5 {
        return;
    }

    let card_rect = Rect {
        x: area.x,
        y: area.y,
        width: area.width.saturating_sub(1),
        height: area.height.saturating_sub(1),
    };
    render_card_shadow(frame, card_rect, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active()))
        .title(
            Line::from(" 📌 INSPECTOR ")
                .bold()
                .style(Style::default().fg(theme.accent())),
        )
        .style(Style::default().bg(theme.card_bg()).fg(theme.text()));

    let (season_name, season_icon) = get_season(selected_date);
    let event_opt = find_event(selected_date);
    let upcoming_opt = next_event(selected_date);
    let is_today = selected_date == today;

    let day_of_year = selected_date.ordinal();
    let total_days = days_in_year(selected_date.year());
    let pct = (day_of_year as f64 / total_days as f64) * 100.0;

    let mut lines = Vec::new();

    // Line 1: Formatted date
    lines.push(Line::from(vec![
        Span::styled("📅 ", Style::default().fg(theme.accent())),
        Span::styled(
            format!(
                "{} {} {}, {}",
                selected_date.day(),
                selected_date.month(),
                selected_date.year(),
                selected_date.weekday()
            ),
            Style::default().fg(theme.text()).bold(),
        ),
        if is_today {
            Span::styled("  [TODAY]", Style::default().fg(theme.accent()).bold())
        } else {
            Span::raw("")
        },
    ]));

    // Line 2: Event status
    if let Some(ev) = event_opt {
        let tag = if ev.is_holiday {
            "Holiday"
        } else {
            "Season / Observance"
        };
        lines.push(Line::from(vec![
            Span::styled(format!("⭐ {} ", ev.icon), Style::default()),
            Span::styled(
                ev.name,
                Style::default()
                    .fg(if ev.is_holiday {
                        theme.holiday_fg()
                    } else {
                        theme.season_fg()
                    })
                    .bold(),
            ),
            Span::styled(format!(" ({tag})"), Style::default().fg(theme.dim())),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("✨ ", Style::default()),
            Span::styled(
                format!("Regular {season_name} Day"),
                Style::default().fg(theme.dim()),
            ),
        ]));
    }

    // Line 3: Season & Quarter
    let quarter = match selected_date.month() as u8 {
        1..=3 => "Q1",
        4..=6 => "Q2",
        7..=9 => "Q3",
        _ => "Q4",
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{season_icon} Season: "),
            Style::default().fg(theme.secondary()),
        ),
        Span::styled(season_name, Style::default().fg(theme.season_fg()).bold()),
        Span::styled(
            format!(" • {quarter} (Quarter)"),
            Style::default().fg(theme.dim()),
        ),
    ]));

    // Line 4: Year Progress with bar
    let bar_len: usize = 10;
    let filled = ((pct / 100.0) * bar_len as f64).round() as usize;
    let empty = bar_len.saturating_sub(filled);
    let bar_str = format!("[{}{}] {:.1}%", "█".repeat(filled), "░".repeat(empty), pct);
    lines.push(Line::from(vec![
        Span::styled("⏳ Year: ", Style::default().fg(theme.primary())),
        Span::styled(bar_str, Style::default().fg(theme.accent()).bold()),
        Span::styled(
            format!(" (Day {day_of_year}/{total_days})"),
            Style::default().fg(theme.dim()),
        ),
    ]));

    // Line 5: Upcoming event
    if let Some((up_ev, days_left)) = upcoming_opt {
        let countdown_str = match days_left {
            0 => "Today!".to_string(),
            1 => "Tomorrow!".to_string(),
            d => format!("in {d} days"),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("🎉 Next: {} {} ", up_ev.icon, up_ev.name),
                Style::default().fg(theme.holiday_fg()),
            ),
            Span::styled(
                format!("({countdown_str})"),
                Style::default().fg(theme.accent()).bold(),
            ),
        ]));
    }

    // Line 6: Color legend
    lines.push(Line::from(vec![
        Span::styled("■ ", Style::default().fg(theme.selected_bg())),
        Span::styled("Selected  ", Style::default().fg(theme.text())),
        Span::styled("● ", Style::default().fg(theme.today_bg())),
        Span::styled("Today  ", Style::default().fg(theme.text())),
        Span::styled("* ", Style::default().fg(theme.holiday_fg()).bold()),
        Span::styled("Holiday  ", Style::default().fg(theme.text())),
        Span::styled("◇ ", Style::default().fg(theme.season_fg())),
        Span::styled("Season", Style::default().fg(theme.text())),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, card_rect);
}

fn render_month_events_card(frame: &mut Frame, area: Rect, month: Month, theme: ThemeKind) {
    if area.width < 5 || area.height < 5 {
        return;
    }

    let card_rect = Rect {
        x: area.x,
        y: area.y,
        width: area.width.saturating_sub(1),
        height: area.height.saturating_sub(1),
    };
    render_card_shadow(frame, card_rect, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border()))
        .title(
            Line::from(format!(" 🌟 {month} EVENTS "))
                .bold()
                .style(Style::default().fg(theme.accent())),
        )
        .style(Style::default().bg(theme.card_bg()).fg(theme.text()));

    let events = events_in_month(month);
    let mut lines = Vec::new();
    if events.is_empty() {
        lines.push(Line::from(
            "  No special holidays listed for this month.".dim(),
        ));
    } else {
        for ev in events {
            let color = if ev.is_holiday {
                theme.holiday_fg()
            } else {
                theme.season_fg()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} Day {:02}: ", ev.icon, ev.day),
                    Style::default().fg(theme.primary()),
                ),
                Span::styled(ev.name, Style::default().fg(color).bold()),
            ]));
        }
    }

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, card_rect);
}

// ==========================================
// Help Modal Overlay with 3D Drop Shadow
// ==========================================

fn render_help_modal(frame: &mut Frame, theme: ThemeKind) {
    let screen = frame.area();
    let modal_area = centered_rect(64, 75, screen);

    let card_rect = Rect {
        x: modal_area.x,
        y: modal_area.y,
        width: modal_area.width.saturating_sub(1),
        height: modal_area.height.saturating_sub(1),
    };

    frame.render_widget(Clear, modal_area);
    render_card_shadow(frame, card_rect, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()).bold())
        .title(
            Line::from(" ⌨️  KEYBOARD SHORTCUTS & HELP ")
                .bold()
                .style(Style::default().fg(theme.accent())),
        )
        .style(Style::default().bg(theme.card_bg()).fg(theme.text()));

    let content = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  h / ←      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Previous day"),
            Span::styled("        j / ↓      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Next week"),
        ]),
        Line::from(vec![
            Span::styled("  l / →      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Next day"),
            Span::styled("            k / ↑      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Previous week"),
        ]),
        Line::from(vec![
            Span::styled("  n / Tab    ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Next month"),
            Span::styled("          p / S-Tab  ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Previous month"),
        ]),
        Line::from(vec![
            Span::styled("  y / ]      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Next year"),
            Span::styled("           Y / [      ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Previous year"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  t          ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Jump to Today (real-world date)"),
        ]),
        Line::from(vec![
            Span::styled("  v / Enter  ", Style::default().fg(theme.secondary()).bold()),
            Span::raw("Toggle View (Year Overview ⟷ Focused Month)"),
        ]),
        Line::from(vec![
            Span::styled("  c          ", Style::default().fg(theme.secondary()).bold()),
            Span::raw("Cycle Theme (B&W Obsidian, B&W Paper, Neobrutalist, Tokyo Night)"),
        ]),
        Line::from(vec![
            Span::styled("  s          ", Style::default().fg(theme.secondary()).bold()),
            Span::raw("Cycle Calendar Format (Headers, Surrounding days, etc.)"),
        ]),
        Line::from(vec![
            Span::styled("  ?          ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Toggle this Help modal"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc    ", Style::default().fg(theme.accent()).bold()),
            Span::raw("Quit Application"),
        ]),
        Line::from(""),
        Line::from(" Press any key or ? to close this help ".dim()).alignment(Alignment::Center),
    ]);

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, card_rect);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}