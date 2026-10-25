use std::fmt;

use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
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
    let mut view_mode = ViewMode::Year;
    loop {
        terminal.draw(|frame| render(frame, calendar_style, selected_date))?;
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Char('s') => calendar_style = calendar_style.next(),
                KeyCode::Char('v') | KeyCode::Enter => view_mode = view_mode.toggle(),
                KeyCode::Char('t') => selected_date = today,
                KeyCode::Char('y') | KeyCode::Char(']') => selected_date = change_year(selected_date, 1),
                KeyCode::Char('Y') | KeyCode::Char('[') => selected_date = change_year(selected_date, -1),
                KeyCode::Char('n') | KeyCode::Tab => selected_date = change_month(selected_date, 1),
                KeyCode::Char('p') | KeyCode::BackTab => selected_date = change_month(selected_date, -1),
                KeyCode::Char('h') | KeyCode::Left => selected_date -= 1.days(),
                KeyCode::Char('j') | KeyCode::Down => selected_date += 1.weeks(),
                KeyCode::Char('k') | KeyCode::Up => selected_date -= 1.weeks(),
                KeyCode::Char('l') | KeyCode::Right => selected_date += 1.days(),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CalendarEvent {
    pub month: Month,
    pub day: u8,
    pub name: &'static str,
    pub icon: &'static str,
    pub is_holiday: bool,
}

pub const CALENDAR_EVENTS: &[CalendarEvent] = &[
    CalendarEvent { month: Month::January, day: 1, name: "New Year's Day", icon: "🎉", is_holiday: true },
    CalendarEvent { month: Month::January, day: 26, name: "Republic Day", icon: "🇮🇳", is_holiday: true },
    CalendarEvent { month: Month::February, day: 2, name: "Groundhog Day", icon: "🦫", is_holiday: false },
    CalendarEvent { month: Month::February, day: 14, name: "Valentine's Day", icon: "💖", is_holiday: false },
    CalendarEvent { month: Month::March, day: 8, name: "Int'l Women's Day", icon: "🌸", is_holiday: false },
    CalendarEvent { month: Month::March, day: 20, name: "Spring Equinox", icon: "🌱", is_holiday: false },
    CalendarEvent { month: Month::April, day: 1, name: "April Fools' Day", icon: "🃏", is_holiday: false },
    CalendarEvent { month: Month::April, day: 22, name: "Earth Day", icon: "🌍", is_holiday: true },
    CalendarEvent { month: Month::May, day: 1, name: "Labor Day", icon: "🛠️", is_holiday: true },
    CalendarEvent { month: Month::May, day: 4, name: "Star Wars Day", icon: "⭐", is_holiday: false },
    CalendarEvent { month: Month::June, day: 21, name: "Summer Solstice", icon: "☀️", is_holiday: false },
    CalendarEvent { month: Month::July, day: 4, name: "Independence Day", icon: "🎆", is_holiday: true },
    CalendarEvent { month: Month::August, day: 15, name: "Independence Day (IN)", icon: "🇮🇳", is_holiday: true },
    CalendarEvent { month: Month::September, day: 22, name: "Autumn Equinox", icon: "🍂", is_holiday: false },
    CalendarEvent { month: Month::October, day: 31, name: "Halloween", icon: "🎃", is_holiday: true },
    CalendarEvent { month: Month::November, day: 11, name: "Veterans Day", icon: "🎖️", is_holiday: true },
    CalendarEvent { month: Month::December, day: 21, name: "Winter Solstice", icon: "❄️", is_holiday: false },
    CalendarEvent { month: Month::December, day: 23, name: "Festivus", icon: "🎄", is_holiday: false },
    CalendarEvent { month: Month::December, day: 24, name: "Christmas Eve", icon: "✨", is_holiday: true },
    CalendarEvent { month: Month::December, day: 25, name: "Christmas Day", icon: "🎁", is_holiday: true },
    CalendarEvent { month: Month::December, day: 31, name: "New Year's Eve", icon: "🥂", is_holiday: true },
];

fn find_event(date: Date) -> Option<&'static CalendarEvent> {
    CALENDAR_EVENTS.iter().find(|e| e.month == date.month() && e.day == date.day())
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

fn days_in_year(year: i32) -> u16 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: Month) -> u8 {
    match month {
        Month::January => 31,
        Month::February => if is_leap_year(year) { 29 } else { 28 },
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

fn render(frame: &mut Frame, calendar_style: StyledCalendar, selected_date: Date) {
    let header = Text::from_iter([
        Line::from("Calendar Example".bold()),
        Line::from(
            "<q> Quit | <s> Change Style | <n> Next Month | <p> Previous Month, <hjkl> Move",
        ),
        Line::from(format!(
            "Current date: {selected_date} | Current style: {calendar_style}"
        )),
    ]);


    let [text_area, area] = frame.area().layout(&Layout::vertical([
        Constraint::Length(header.height() as u16),
        Constraint::Fill(1),
    ]));

    frame.render_widget(header.centered(), text_area);
    calendar_style
        .render_year(frame, area, selected_date)
        .unwrap();
}

#[derive(Debug, Clone, Copy)]
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

enum StyledCalendar {
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
            Self::Default => Self::Surrounding,
            Self::Surrounding => Self::WeekdaysHeader,
            Self::WeekdaysHeader => Self::SurroundingAndWeekdaysHeader,
            Self::SurroundingAndWeekdaysHeader => Self::MonthHeader,
            Self::MonthHeader => Self::MonthAndWeekdaysHeader,
            Self::MonthAndWeekdaysHeader => Self::Default,
        }
    }
}

impl fmt::Display for StyledCalendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "Default"),
            Self::Surrounding => write!(f, "Show Surrounding"),
            Self::WeekdaysHeader => write!(f, "Show Weekdays Header"),
            Self::SurroundingAndWeekdaysHeader => write!(f, "Show Surrounding and Weekdays Header"),
            Self::MonthHeader => write!(f, "Show Month Header"),
            Self::MonthAndWeekdaysHeader => write!(f, "Show Month Header and Weekdays Header"),
        }
    }
}

impl StyledCalendar {
    fn render_year(self, frame: &mut Frame, area: Rect, date: Date) -> Result<()> {
        let events = events(date)?;

        let vertical = Layout::vertical([Constraint::Ratio(1, 3); 3]);
        let horizontal = &Layout::horizontal([Constraint::Ratio(1, 4); 4]);
        let areas = area
            .inner(Margin::new(1, 1))
            .layout_vec(&vertical)
            .into_iter()
            .flat_map(|row| row.layout_vec(horizontal));
        for (i, area) in areas.enumerate() {
            let month = date
                .replace_day(1)
                .unwrap()
                .replace_month(Month::try_from(i as u8 + 1).unwrap())
                .unwrap();
            self.render_month(frame, area, month, &events);
        }
        Ok(())
    }

    fn render_month(self, frame: &mut Frame, area: Rect, date: Date, events: &CalendarEventStore) {
        let calendar = match self {
            Self::Default => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default()),
            Self::Surrounding => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default())
                .show_surrounding(Style::new().dim()),
            Self::WeekdaysHeader => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default())
                .show_weekdays_header(Style::new().bold().green()),
            Self::SurroundingAndWeekdaysHeader => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default())
                .show_surrounding(Style::new().dim())
                .show_weekdays_header(Style::new().bold().green()),
            Self::MonthHeader => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default())
                .show_month_header(Style::new().bold().green()),
            Self::MonthAndWeekdaysHeader => Monthly::new(date, events)
                .default_style(Style::new().bold().bg(Color::Rgb(50, 50, 50)))
                .show_month_header(Style::default())
                .show_weekdays_header(Style::new().bold().dim().light_yellow()),
        };
        frame.render_widget(calendar, area);
    }
}

fn events(selected_date: Date) -> Result<CalendarEventStore> {
    const SELECTED: Style = Style::new()
        .fg(Color::White)
        .bg(Color::Red)
        .add_modifier(Modifier::BOLD);
    const HOLIDAY: Style = Style::new()
        .fg(Color::Red)
        .add_modifier(Modifier::UNDERLINED);
    const SEASON: Style = Style::new()
        .fg(Color::Green)
        .bg(Color::Black)
        .add_modifier(Modifier::UNDERLINED);

    let mut list = CalendarEventStore::today(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Blue),
    );
    let y = selected_date.year();

    list.add(Date::from_calendar_date(y, Month::January, 1)?, HOLIDAY);
    list.add(Date::from_calendar_date(y + 1, Month::January, 1)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::February, 2)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::April, 1)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::April, 22)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::May, 4)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::December, 23)?, HOLIDAY);
    list.add(Date::from_calendar_date(y, Month::December, 31)?, HOLIDAY);

    list.add(Date::from_calendar_date(y, Month::March, 22)?, SEASON);
    list.add(Date::from_calendar_date(y, Month::June, 21)?, SEASON);
    list.add(Date::from_calendar_date(y, Month::September, 22)?, SEASON);
    list.add(Date::from_calendar_date(y, Month::December, 21)?, SEASON);

    list.add(selected_date, SELECTED);

    Ok(list)
}
