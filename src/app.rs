use chrono::{DateTime, Datelike, Local, NaiveDate};
use iced::theme::Palette;
use iced::time;
use iced::widget::text_editor;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, scrollable, text,
    text_editor as editor, text_input,
};
use iced::{Alignment, Color, Element, Length, Subscription, Task, Theme, window};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

use crate::models::{Category, Session};
use crate::repository::{Repository, SqliteRepository};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Timer,
    Overview,
    History,
    Calendar,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerStatus {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    const ALL: [ThemePreference; 3] = [
        ThemePreference::System,
        ThemePreference::Light,
        ThemePreference::Dark,
    ];

    fn from_db(value: &str) -> Self {
        match value {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => ThemePreference::System,
        }
    }

    fn as_db(self) -> &'static str {
        match self {
            ThemePreference::System => "system",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }
}

impl Display for ThemePreference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemePreference::System => write!(f, "System"),
            ThemePreference::Light => write!(f, "Light"),
            ThemePreference::Dark => write!(f, "Dark"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CategoryChoice {
    id: i64,
    name: String,
}

impl Display for CategoryChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
struct TimerSession {
    session_id: Option<i64>,
    category_id: Option<i64>,
    session_name: String,
    accumulated_before_run: i64,
    running_since: Option<Instant>,
    status: TimerStatus,
}

impl Default for TimerSession {
    fn default() -> Self {
        Self {
            session_id: None,
            category_id: None,
            session_name: String::new(),
            accumulated_before_run: 0,
            running_since: None,
            status: TimerStatus::Stopped,
        }
    }
}

impl TimerSession {
    fn elapsed_seconds(&self) -> i64 {
        let mut total = self.accumulated_before_run;
        if self.status == TimerStatus::Running
            && let Some(started) = self.running_since
        {
            total += started.elapsed().as_secs() as i64;
        }
        total
    }

    fn pause(&mut self) {
        if self.status == TimerStatus::Running
            && let Some(started) = self.running_since.take()
        {
            self.accumulated_before_run += started.elapsed().as_secs() as i64;
        }
        self.status = TimerStatus::Paused;
    }

    fn start(&mut self) {
        if self.status != TimerStatus::Running {
            self.status = TimerStatus::Running;
            self.running_since = Some(Instant::now());
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone)]
enum Message {
    ChangePage(Page),
    Tick,
    CategoryPicked(CategoryChoice),
    ToggleNewCategoryInput,
    CancelAddCategory,
    NewCategoryChanged(String),
    AddCategory,
    SessionNameChanged(String),
    NotesEdited(text_editor::Action),
    HistoryNotesEdited(i64, text_editor::Action),
    StartPressed,
    PausePressed,
    StopPressed,
    RequestDeleteCategory(i64),
    ConfirmDeleteCategory(i64),
    CancelDeleteCategory,
    ToggleNotes(i64),
    DeleteSession(i64),
    SelectPreviousMonth,
    SelectNextMonth,
    SelectPreviousYear,
    SelectNextYear,
    SelectToday,
    SelectCalendarDay(u32),
    ThemeChanged(ThemePreference),
    PickExportPath,
    PickImportPath,
}

#[derive(Clone, Copy)]
struct AppColors {
    primary: Color,
    on_primary: Color,
    secondary: Color,
    tertiary: Color,
    error: Color,
    surface: Color,
    on_surface: Color,
    surface_variant: Color,
    outline: Color,
    hover: Color,
}

const DARK_COLORS: AppColors = AppColors {
    primary: color_hex(0xa2b574),
    on_primary: color_hex(0x161311),
    secondary: color_hex(0xdfb26c),
    tertiary: color_hex(0xe09260),
    error: color_hex(0xe37874),
    surface: color_hex(0x161311),
    on_surface: color_hex(0xe6dfd3),
    surface_variant: color_hex(0x26211e),
    outline: color_hex(0x453a35),
    hover: color_hex(0xa2b574),
};

const LIGHT_COLORS: AppColors = AppColors {
    primary: color_hex(0x4b5e37),
    on_primary: color_hex(0xe5d8be),
    secondary: color_hex(0x8c6f39),
    tertiary: color_hex(0xa86439),
    error: color_hex(0xa6464b),
    surface: color_hex(0xe5d8be),
    on_surface: color_hex(0x2d2622),
    surface_variant: color_hex(0xd1c5b2),
    outline: color_hex(0x9e8e7a),
    hover: color_hex(0x4b5e37),
};

const fn color_hex(rgb: u32) -> Color {
    let r = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb & 0xff) as f32 / 255.0;
    Color::from_rgb(r, g, b)
}

pub struct SkillTrackApp {
    repo: SqliteRepository,
    page: Page,
    categories: Vec<Category>,
    sessions: Vec<Session>,
    totals: HashMap<i64, i64>,
    selected_category_id: Option<i64>,
    selected_date: NaiveDate,
    expanded_notes: HashSet<i64>,
    new_category_name: String,
    show_new_category_input: bool,
    confirm_delete_category_id: Option<i64>,
    timer: TimerSession,
    notes_content: text_editor::Content,
    history_notes: HashMap<i64, text_editor::Content>,
    dirty_history_notes: HashSet<i64>,
    theme_preference: ThemePreference,
    export_path: String,
    import_path: String,
    status_message: Option<String>,
}

impl SkillTrackApp {
    fn new(repo: SqliteRepository) -> (Self, Task<Message>) {
        let mut app = Self {
            repo,
            page: Page::Timer,
            categories: Vec::new(),
            sessions: Vec::new(),
            totals: HashMap::new(),
            selected_category_id: None,
            selected_date: Local::now().date_naive(),
            expanded_notes: HashSet::new(),
            new_category_name: String::new(),
            show_new_category_input: false,
            confirm_delete_category_id: None,
            timer: TimerSession::default(),
            notes_content: text_editor::Content::new(),
            history_notes: HashMap::new(),
            dirty_history_notes: HashSet::new(),
            theme_preference: ThemePreference::System,
            export_path: String::new(),
            import_path: String::new(),
            status_message: None,
        };

        app.load_theme();
        if let Err(err) = app.refresh_data() {
            app.set_error("Failed to load initial data", &err);
        }

        (app, Task::none())
    }

    pub fn run(repo: SqliteRepository) -> iced::Result {
        let icon = window::icon::from_file_data(include_bytes!("icon.png"), None).ok();
        let window_settings = window::Settings {
            icon,
            ..window::Settings::default()
        };

        iced::application("rhizometrack", Self::update, Self::view)
            .window(window_settings)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .run_with(|| Self::new(repo))
    }

    fn load_theme(&mut self) {
        match self.repo.get_theme() {
            Ok(Some(saved)) => {
                self.theme_preference = ThemePreference::from_db(saved.as_str());
            }
            Ok(None) => {}
            Err(err) => self.set_error("Failed to load theme setting", &err),
        }
    }

    fn set_error(&mut self, context: &str, err: &dyn Display) {
        self.status_message = Some(format!("{context}: {err}"));
    }

    fn set_info(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
    }

    fn clear_status(&mut self) {
        self.status_message = None;
    }

    fn current_notes_text(&self) -> String {
        self.notes_content.text().trim_end_matches('\n').to_string()
    }

    fn active_colors(&self) -> Option<AppColors> {
        match self.theme_preference {
            ThemePreference::Dark => Some(DARK_COLORS),
            ThemePreference::Light => Some(LIGHT_COLORS),
            ThemePreference::System => None,
        }
    }

    fn refresh_data(&mut self) -> Result<(), String> {
        self.categories = self
            .repo
            .all_categories()
            .map_err(|e| format!("loading categories failed: {e}"))?;
        self.categories
            .sort_by(|left, right| right.created_at.cmp(&left.created_at));

        self.sessions = self
            .repo
            .all_sessions()
            .map_err(|e| format!("loading sessions failed: {e}"))?;

        let totals = self
            .repo
            .category_totals()
            .map_err(|e| format!("loading totals failed: {e}"))?;
        self.totals.clear();
        for item in totals {
            self.totals.insert(item.category_id, item.total_seconds);
        }

        if let Some(selected) = self.selected_category_id
            && !self.categories.iter().any(|cat| cat.id == selected)
        {
            self.selected_category_id = None;
        }

        Ok(())
    }

    fn flush_history_note_saves(&mut self) {
        let session_ids: Vec<i64> = self.dirty_history_notes.iter().copied().collect();
        for session_id in session_ids {
            let Some(content) = self.history_notes.get(&session_id) else {
                self.dirty_history_notes.remove(&session_id);
                continue;
            };
            let notes = content.text().trim_end_matches('\n').to_string();
            match self.repo.update_session_notes(session_id, notes.as_str()) {
                Ok(()) => {
                    if let Some(session) = self.sessions.iter_mut().find(|s| s.id == session_id) {
                        session.note_markdown = notes;
                    }
                    self.dirty_history_notes.remove(&session_id);
                }
                Err(err) => {
                    self.set_error("Auto-save of history notes failed", &err);
                    break;
                }
            }
        }
    }

    fn category_choices(&self) -> Vec<CategoryChoice> {
        self.categories
            .iter()
            .map(|cat| CategoryChoice {
                id: cat.id,
                name: cat.name.clone(),
            })
            .collect()
    }

    fn selected_category_choice(&self) -> Option<CategoryChoice> {
        let selected = self.selected_category_id?;
        self.categories
            .iter()
            .find(|cat| cat.id == selected)
            .map(|cat| CategoryChoice {
                id: cat.id,
                name: cat.name.clone(),
            })
    }

    fn current_seconds_for_category(&self, category_id: i64) -> i64 {
        let mut total = *self.totals.get(&category_id).unwrap_or(&0);
        if self.timer.session_id.is_some()
            && self.timer.category_id == Some(category_id)
            && self.timer.status != TimerStatus::Stopped
        {
            total += self.timer.elapsed_seconds();
        }
        total
    }

    fn format_hms(seconds: i64) -> String {
        let clamped = seconds.max(0);
        let s = clamped % 60;
        let m = (clamped / 60) % 60;
        let h = clamped / 3600;
        format!("{h:02}:{m:02}:{s:02}")
    }

    fn category_name(&self, id: i64) -> String {
        self.categories
            .iter()
            .find(|cat| cat.id == id)
            .map(|cat| cat.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn switch_active_category(&mut self, new_category_id: i64) {
        if self.timer.status == TimerStatus::Stopped || self.timer.session_id.is_none() {
            self.selected_category_id = Some(new_category_id);
            self.timer.category_id = Some(new_category_id);
            return;
        }

        if self.timer.category_id == Some(new_category_id) {
            self.selected_category_id = Some(new_category_id);
            return;
        }

        if self.timer.status == TimerStatus::Running {
            self.timer.pause();
        }

        let Some(old_session_id) = self.timer.session_id else {
            self.selected_category_id = Some(new_category_id);
            return;
        };

        let duration = self.timer.elapsed_seconds();
        let old_notes = self.current_notes_text();
        let old_name = self.timer.session_name.clone();
        let old_name_param = if old_name.trim().is_empty() {
            None
        } else {
            Some(old_name.as_str())
        };

        if let Err(err) =
            self.repo
                .finish_session(old_session_id, duration, old_notes.as_str(), old_name_param)
        {
            self.set_error("Failed to finish old session during category switch", &err);
            return;
        }

        let new_name_param = if self.timer.session_name.trim().is_empty() {
            None
        } else {
            Some(self.timer.session_name.as_str())
        };

        match self.repo.create_session(new_category_id, new_name_param) {
            Ok(new_session_id) => {
                self.timer.session_id = Some(new_session_id);
                self.timer.category_id = Some(new_category_id);
                self.timer.accumulated_before_run = 0;
                self.timer.running_since = None;
                self.timer.status = TimerStatus::Paused;
                self.selected_category_id = Some(new_category_id);
                self.notes_content = text_editor::Content::new();
                self.history_notes.clear();
                self.dirty_history_notes.clear();
                if let Err(err) = self.refresh_data() {
                    self.set_error("Failed to refresh data after category switch", &err);
                }
            }
            Err(err) => {
                self.set_error("Failed to create new session during category switch", &err);
                self.timer.reset();
                self.selected_category_id = Some(new_category_id);
                let _ = self.refresh_data();
            }
        }
    }

    fn start_timer(&mut self) {
        let Some(category_id) = self.selected_category_id else {
            self.set_info("Select a category before you start the timer.");
            return;
        };

        if self.timer.status == TimerStatus::Running {
            return;
        }

        if self.timer.session_id.is_none() {
            let session_name_param = if self.timer.session_name.trim().is_empty() {
                None
            } else {
                Some(self.timer.session_name.as_str())
            };
            match self.repo.create_session(category_id, session_name_param) {
                Ok(session_id) => {
                    self.timer.session_id = Some(session_id);
                    self.timer.category_id = Some(category_id);
                    self.timer.accumulated_before_run = 0;
                    self.notes_content = text_editor::Content::new();
                }
                Err(err) => {
                    self.set_error("Failed to create session", &err);
                    return;
                }
            }
        } else {
            self.timer.category_id = Some(category_id);
        }

        self.timer.start();
        self.clear_status();
    }

    fn pause_timer(&mut self) {
        if self.timer.status == TimerStatus::Running {
            self.timer.pause();
        }
    }

    fn stop_timer(&mut self) {
        if self.timer.status == TimerStatus::Stopped {
            return;
        }

        if self.timer.status == TimerStatus::Running {
            self.timer.pause();
        }

        if let Some(session_id) = self.timer.session_id {
            let duration = self.timer.elapsed_seconds();
            let notes = self.current_notes_text();
            let session_name_param = if self.timer.session_name.trim().is_empty() {
                None
            } else {
                Some(self.timer.session_name.as_str())
            };

            if let Err(err) =
                self.repo
                    .finish_session(session_id, duration, notes.as_str(), session_name_param)
            {
                self.set_error("Failed to finish session", &err);
                return;
            }
        }

        self.timer.reset();
        self.notes_content = text_editor::Content::new();
        if let Err(err) = self.refresh_data() {
            self.set_error("Failed to refresh data after stopping timer", &err);
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangePage(next) => {
                self.page = next;
                self.confirm_delete_category_id = None;
                self.clear_status();
            }
            Message::Tick => {
                self.flush_history_note_saves();
            }
            Message::CategoryPicked(choice) => self.switch_active_category(choice.id),
            Message::ToggleNewCategoryInput => {
                self.show_new_category_input = true;
            }
            Message::CancelAddCategory => {
                self.show_new_category_input = false;
                self.new_category_name.clear();
            }
            Message::NewCategoryChanged(value) => self.new_category_name = value,
            Message::AddCategory => {
                let name = self.new_category_name.trim().to_string();
                if name.is_empty() {
                    self.set_info("Category name cannot be empty.");
                    return Task::none();
                }
                match self.repo.create_category(name.as_str(), None) {
                    Ok(new_id) => {
                        self.new_category_name.clear();
                        self.show_new_category_input = false;
                        self.selected_category_id = Some(new_id);
                        self.timer.category_id = Some(new_id);
                        if let Err(err) = self.refresh_data() {
                            self.set_error("Failed to refresh data after category creation", &err);
                        }
                    }
                    Err(err) => self.set_error("Failed to create category", &err),
                }
            }
            Message::SessionNameChanged(value) => self.timer.session_name = value,
            Message::NotesEdited(action) => {
                self.notes_content.perform(action);
            }
            Message::HistoryNotesEdited(session_id, action) => {
                if let Some(content) = self.history_notes.get_mut(&session_id) {
                    content.perform(action);
                    self.dirty_history_notes.insert(session_id);
                }
            }
            Message::StartPressed => self.start_timer(),
            Message::PausePressed => self.pause_timer(),
            Message::StopPressed => self.stop_timer(),
            Message::RequestDeleteCategory(category_id) => {
                self.confirm_delete_category_id = Some(category_id);
                self.set_info("Press Confirm to delete this category.");
            }
            Message::ConfirmDeleteCategory(category_id) => {
                if self.confirm_delete_category_id != Some(category_id) {
                    self.set_info("Select delete on the category first.");
                    return Task::none();
                }
                match self.repo.delete_category(category_id) {
                    Ok(()) => {
                        self.confirm_delete_category_id = None;
                        if self.selected_category_id == Some(category_id) {
                            self.selected_category_id = None;
                        }
                        if self.timer.category_id == Some(category_id) {
                            self.timer.category_id = None;
                        }
                        if let Err(err) = self.refresh_data() {
                            self.set_error("Failed to refresh data after category deletion", &err);
                        }
                    }
                    Err(err) => self.set_error("Failed to delete category", &err),
                }
            }
            Message::CancelDeleteCategory => {
                self.confirm_delete_category_id = None;
                self.set_info("Category delete canceled.");
            }
            Message::ToggleNotes(session_id) => {
                if !self.expanded_notes.insert(session_id) {
                    self.expanded_notes.remove(&session_id);
                    self.dirty_history_notes.remove(&session_id);
                } else if !self.history_notes.contains_key(&session_id)
                    && let Some(session) = self.sessions.iter().find(|entry| entry.id == session_id)
                {
                    self.history_notes.insert(
                        session_id,
                        text_editor::Content::with_text(session.note_markdown.as_str()),
                    );
                }
            }
            Message::DeleteSession(session_id) => match self.repo.delete_session(session_id) {
                Ok(()) => {
                    self.expanded_notes.remove(&session_id);
                    self.history_notes.remove(&session_id);
                    self.dirty_history_notes.remove(&session_id);
                    if let Err(err) = self.refresh_data() {
                        self.set_error("Failed to refresh data after session deletion", &err);
                    }
                }
                Err(err) => self.set_error("Failed to delete session", &err),
            },
            Message::SelectPreviousMonth => {
                self.selected_date = shift_month(self.selected_date, -1);
            }
            Message::SelectNextMonth => {
                self.selected_date = shift_month(self.selected_date, 1);
            }
            Message::SelectPreviousYear => {
                self.selected_date = shift_year(self.selected_date, -1);
            }
            Message::SelectNextYear => {
                self.selected_date = shift_year(self.selected_date, 1);
            }
            Message::SelectToday => {
                self.selected_date = Local::now().date_naive();
            }
            Message::SelectCalendarDay(day) => {
                if let Some(date) = NaiveDate::from_ymd_opt(
                    self.selected_date.year(),
                    self.selected_date.month(),
                    day,
                ) {
                    self.selected_date = date;
                }
            }
            Message::ThemeChanged(next) => {
                self.theme_preference = next;
                if let Err(err) = self.repo.set_theme(next.as_db()) {
                    self.set_error("Failed to save theme setting", &err);
                }
            }
            Message::PickExportPath => {
                let selection = rfd::FileDialog::new()
                    .set_file_name("rhizometrack-export.json")
                    .save_file();
                if let Some(path) = selection {
                    self.export_path = path.display().to_string();
                    match self.repo.export_data() {
                        Ok(data) => match serde_json::to_string_pretty(&data) {
                            Ok(json) => match std::fs::write(&self.export_path, json) {
                                Ok(()) => self.set_info("Export completed."),
                                Err(err) => self.set_error("Failed to write export file", &err),
                            },
                            Err(err) => self.set_error("Failed to serialize export data", &err),
                        },
                        Err(err) => self.set_error("Failed to gather export data", &err),
                    }
                }
            }
            Message::PickImportPath => {
                let selection = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file();
                if let Some(path) = selection {
                    self.import_path = path.display().to_string();
                    match std::fs::read_to_string(&self.import_path) {
                        Ok(raw) => match serde_json::from_str(raw.as_str()) {
                            Ok(data) => match self.repo.import_data(data) {
                                Ok(()) => {
                                    if let Err(err) = self.refresh_data() {
                                        self.set_error("Import completed but refresh failed", &err);
                                    } else {
                                        self.set_info("Import completed.");
                                    }
                                }
                                Err(err) => self.set_error("Failed to import data", &err),
                            },
                            Err(err) => self.set_error("Invalid import JSON file", &err),
                        },
                        Err(err) => self.set_error("Failed to read import file", &err),
                    }
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        match self.theme_preference {
            ThemePreference::System => Theme::default(),
            ThemePreference::Light => Theme::custom(
                "rhizometrack Light".to_string(),
                Palette {
                    background: LIGHT_COLORS.surface,
                    text: LIGHT_COLORS.on_surface,
                    primary: LIGHT_COLORS.primary,
                    success: LIGHT_COLORS.secondary,
                    danger: LIGHT_COLORS.error,
                },
            ),
            ThemePreference::Dark => Theme::custom(
                "rhizometrack Dark".to_string(),
                Palette {
                    background: DARK_COLORS.surface,
                    text: DARK_COLORS.on_surface,
                    primary: DARK_COLORS.primary,
                    success: DARK_COLORS.secondary,
                    danger: DARK_COLORS.error,
                },
            ),
        }
    }

    fn page_container<'a>(&self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        let mut base = container(content.into())
            .width(Length::Fill)
            .center_x(Length::Fill);

        if let Some(colors) = self.active_colors() {
            base = base.style(move |_| {
                container::Style::default()
                    .background(colors.surface_variant)
                    .color(colors.on_surface)
                    .border(iced::Border {
                        color: colors.outline,
                        width: 1.0,
                        radius: 12.0.into(),
                    })
            });
        }

        base.padding(16).max_width(1280).into()
    }

    fn normal_button<'a>(
        &self,
        label: &'a str,
        message: Message,
    ) -> iced::widget::Button<'a, Message> {
        let mut btn = button(label).on_press(message);
        if let Some(colors) = self.active_colors() {
            btn = btn.style(move |_, status| {
                let mut style = iced::widget::button::Style {
                    background: Some(iced::Background::Color(colors.primary)),
                    text_color: colors.on_primary,
                    border: iced::Border {
                        color: colors.outline,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                };
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(colors.hover));
                }
                style
            });
        }
        btn
    }

    fn danger_button<'a>(
        &self,
        label: &'a str,
        message: Message,
    ) -> iced::widget::Button<'a, Message> {
        let mut btn = button(label).on_press(message);
        if let Some(colors) = self.active_colors() {
            btn = btn.style(move |_, status| {
                let mut style = iced::widget::button::Style {
                    background: Some(iced::Background::Color(colors.error)),
                    text_color: colors.on_primary,
                    border: iced::Border {
                        color: colors.outline,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                };
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(colors.tertiary));
                }
                style
            });
        }
        btn
    }

    fn view(&self) -> Element<'_, Message> {
        let nav = row![
            self.normal_button("Timer", Message::ChangePage(Page::Timer)),
            self.normal_button("Stats", Message::ChangePage(Page::Overview)),
            self.normal_button("Notes", Message::ChangePage(Page::History)),
            self.normal_button("Calendar", Message::ChangePage(Page::Calendar)),
            self.normal_button("Settings", Message::ChangePage(Page::Settings)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let mut content = column![container(nav).center_x(Length::Fill)]
            .spacing(14)
            .align_x(Alignment::Center);
        if let Some(message) = &self.status_message {
            content = content.push(text(message));
        }

        let body = match self.page {
            Page::Timer => self.view_timer_page(),
            Page::Overview => self.view_overview_page(),
            Page::History => self.view_history_page(),
            Page::Calendar => self.view_calendar_page(),
            Page::Settings => self.view_settings_page(),
        };

        content = content.push(body);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(16)
            .into()
    }

    fn view_timer_page(&self) -> Element<'_, Message> {
        let category_row = row![
            pick_list(
                self.category_choices(),
                self.selected_category_choice(),
                Message::CategoryPicked
            )
            .placeholder("Select category")
            .width(Length::Fixed(380.0)),
            self.normal_button("+", Message::ToggleNewCategoryInput),
        ]
        .width(Length::Fixed(420.0))
        .spacing(8)
        .align_y(Alignment::Center);

        let add_category_row = if self.show_new_category_input {
            row![
                text_input("New category", &self.new_category_name)
                    .on_input(Message::NewCategoryChanged)
                    .padding(8)
                    .width(Length::Fixed(220.0)),
                self.normal_button("Create", Message::AddCategory),
                self.normal_button("Cancel", Message::CancelAddCategory),
            ]
            .width(Length::Fixed(420.0))
            .spacing(8)
            .align_y(Alignment::Center)
        } else {
            row![]
        };

        let timer_label = text(Self::format_hms(self.timer.elapsed_seconds())).size(52);
        let start_label = if self.timer.status == TimerStatus::Paused {
            "Resume"
        } else {
            "Start"
        };

        let controls = row![
            self.normal_button(start_label, Message::StartPressed),
            self.normal_button("Pause", Message::PausePressed),
            self.normal_button("Stop", Message::StopPressed),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let notes_box = editor(&self.notes_content)
            .placeholder("Session notes")
            .on_action(Message::NotesEdited)
            .height(Length::Fill)
            .padding(10);

        let page = column![
            text("Timer").size(34),
            category_row,
            add_category_row,
            text_input("Session name (optional)", &self.timer.session_name)
                .on_input(Message::SessionNameChanged)
                .padding(8)
                .width(Length::Fixed(420.0)),
            timer_label,
            controls,
            container(notes_box)
                .width(Length::Fill)
                .max_width(980)
                .height(Length::Fill),
        ]
        .spacing(14)
        .align_x(Alignment::Center)
        .height(Length::Fill)
        .width(Length::Fill);

        self.page_container(page)
    }

    fn view_overview_page(&self) -> Element<'_, Message> {
        let mut items = column![text("Stats").size(34)]
            .spacing(12)
            .align_x(Alignment::Center);

        let mut ranked_categories: Vec<(&Category, i64, i64, i64, i64)> = self
            .categories
            .iter()
            .map(|category| {
                let seconds = self.current_seconds_for_category(category.id);
                let xp = crate::xp::seconds_to_xp(seconds);
                let (level, progress, needed) = crate::xp::level_for_xp(xp);
                (category, seconds, level, progress, needed)
            })
            .collect();

        ranked_categories.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| b.3.cmp(&a.3))
        });

        for (category, seconds, level, progress, needed) in ranked_categories {
            let fraction = if needed > 0 {
                progress as f32 / needed as f32
            } else {
                0.0
            };

            let mut header_row = row![
                text(format!("{} (Lv {})", category.name, level)),
                text(Self::format_hms(seconds)),
            ]
            .spacing(12)
            .align_y(Alignment::Center);

            if self.confirm_delete_category_id == Some(category.id) {
                header_row = header_row
                    .push(self.normal_button("Confirm", Message::ConfirmDeleteCategory(category.id)))
                    .push(self.normal_button("Cancel", Message::CancelDeleteCategory));
            } else {
                header_row =
                    header_row.push(self.danger_button("Delete", Message::RequestDeleteCategory(category.id)));
            }

            let row_item = column![
                header_row,
                progress_bar(0.0..=1.0, fraction).width(Length::Fill),
            ]
            .spacing(8)
            .align_x(Alignment::Center);

            items = items.push(container(row_item).width(Length::Fill).padding(10));
        }

        let stats_panel = self.page_container(scrollable(items));
        let (longest_time, longest_detail) = self.longest_session_parts();
        let extra_stats = container(
            column![self.stat_badge(longest_time, longest_detail)]
                .align_x(Alignment::Start)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .max_width(1280)
        .padding([4, 16]);

        column![stats_panel, extra_stats]
            .spacing(10)
            .width(Length::Fill)
            .into()
    }

    fn longest_session_parts(&self) -> (String, String) {
        if let Some(session) = self
            .sessions
            .iter()
            .max_by_key(|session| session.duration_seconds)
        {
            let date = parse_rfc3339_date(session.started_at.as_str())
                .map(|d| d.to_string())
                .unwrap_or_else(|| "unknown-date".to_string());
            let name = if session.session_name.trim().is_empty() {
                "(no session name)"
            } else {
                session.session_name.as_str()
            };

            let name_chars = name.chars().count();
            let short_name = if name_chars > 24 {
                let clipped: String = name.chars().take(24).collect();
                format!("{clipped}...")
            } else {
                name.to_string()
            };
            (
                Self::format_hms(session.duration_seconds),
                format!(
                    "{} - {}",
                    parse_rfc3339_date(session.started_at.as_str())
                        .map(|d| d.format("%d/%m/%y").to_string())
                        .unwrap_or(date),
                    short_name
                ),
            )
        } else {
            ("--:--:--".to_string(), "no sessions yet".to_string())
        }
    }

    fn stat_badge<'a>(&self, longest_time: String, longest_detail: String) -> iced::widget::Button<'a, Message> {
        let mut time_text = text(longest_time).size(38);
        if let Some(colors) = self.active_colors() {
            time_text = time_text.color(colors.primary);
        }

        let badge_content = column![
            text("Longest session").size(18),
            time_text,
            text(longest_detail).size(16),
        ]
        .align_x(Alignment::Center)
        .spacing(6);

        let mut badge = button(
            container(badge_content)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fixed(260.0))
        .height(Length::Fixed(140.0))
        .padding([8, 10]);
        if let Some(colors) = self.active_colors() {
            badge = badge.style(move |_, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    colors.secondary.r,
                    colors.secondary.g,
                    colors.secondary.b,
                    0.28,
                ))),
                text_color: colors.on_surface,
                border: iced::Border {
                    color: colors.outline,
                    width: 2.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(colors.outline.r, colors.outline.g, colors.outline.b, 0.35),
                    offset: iced::Vector::new(0.0, 3.0),
                    blur_radius: 8.0,
                },
            });
        }
        badge
    }

    fn view_history_page(&self) -> Element<'_, Message> {
        let mut items = column![
            text("Notes").size(34),
            text(format!("Filter date: {}", self.selected_date)),
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        for session in &self.sessions {
            let Some(parsed_date) = parse_rfc3339_date(session.started_at.as_str()) else {
                continue;
            };
            if parsed_date != self.selected_date {
                continue;
            }

            let title = if session.session_name.trim().is_empty() {
                format!(
                    "{} - {}",
                    self.category_name(session.category_id),
                    Self::format_hms(session.duration_seconds)
                )
            } else {
                format!(
                    "{}: {} - {}",
                    self.category_name(session.category_id),
                    session.session_name,
                    Self::format_hms(session.duration_seconds)
                )
            };
            let show_notes = self.expanded_notes.contains(&session.id);
            let toggle = if show_notes {
                "Hide notes"
            } else {
                "Show notes"
            };

            let mut row_content = column![
                row![
                    text(title),
                    self.normal_button(toggle, Message::ToggleNotes(session.id)),
                    self.danger_button("Delete", Message::DeleteSession(session.id)),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            ]
            .spacing(8)
            .align_x(Alignment::Center);

            if show_notes {
                if let Some(content) = self.history_notes.get(&session.id) {
                    row_content = row_content
                        .push(
                            container(
                                editor(content)
                                    .on_action(move |action| {
                                        Message::HistoryNotesEdited(session.id, action)
                                    })
                                    .height(Length::Fixed(160.0))
                                    .padding(8),
                            )
                            .width(Length::Fixed(620.0)),
                        )
                } else {
                    row_content = row_content.push(text("Open notes again to edit."));
                }
            }

            items = items.push(container(row_content).padding(8));
        }

        self.page_container(scrollable(items))
    }

    fn view_calendar_page(&self) -> Element<'_, Message> {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let days_total = days_in_month(year, month);
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(self.selected_date);
        let offset = first_of_month.weekday().num_days_from_monday() as usize;
        let marked_days = self.marked_days_for_current_month();

        let mut day_rows = column![
            row![
                text("Mon"),
                text("Tue"),
                text("Wed"),
                text("Thu"),
                text("Fri"),
                text("Sat"),
                text("Sun")
            ]
            .spacing(8)
        ]
        .spacing(8)
        .align_x(Alignment::Center);

        let mut day_number = 1u32;
        for week in 0..6 {
            let mut week_row = row![].spacing(8).align_y(Alignment::Center);
            for day_idx in 0..7 {
                let idx = week * 7 + day_idx;
                if idx < offset || day_number > days_total {
                    week_row = week_row.push(container(text(" ")).width(Length::Fixed(68.0)));
                } else {
                    let has_session = marked_days.contains(&day_number);
                    let label = if has_session {
                        format!("{} •", day_number)
                    } else {
                        day_number.to_string()
                    };
                    let selected = day_number == self.selected_date.day();
                    let button = self.calendar_day_button(label, day_number, selected, has_session);
                    week_row = week_row.push(button.width(Length::Fixed(68.0)));
                    day_number += 1;
                }
            }
            day_rows = day_rows.push(week_row);
            if day_number > days_total {
                break;
            }
        }

        let page = column![
            text("Calendar").size(34),
            row![
                self.normal_button("<< Year", Message::SelectPreviousYear),
                self.normal_button("< Month", Message::SelectPreviousMonth),
                self.normal_button("Today", Message::SelectToday),
                self.normal_button("Month >", Message::SelectNextMonth),
                self.normal_button("Year >>", Message::SelectNextYear),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(self.selected_date.format("%B %Y").to_string()),
            day_rows,
            text(format!("Selected date: {}", self.selected_date)),
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        self.page_container(page)
    }

    fn marked_days_for_current_month(&self) -> HashSet<u32> {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        self.sessions
            .iter()
            .filter_map(|session| parse_rfc3339_date(session.started_at.as_str()))
            .filter(|date| date.year() == year && date.month() == month)
            .map(|date| date.day())
            .collect()
    }

    fn calendar_day_button<'a>(
        &self,
        label: String,
        day: u32,
        selected: bool,
        has_session: bool,
    ) -> iced::widget::Button<'a, Message> {
        let mut btn = button(text(label).size(14)).on_press(Message::SelectCalendarDay(day));
        if let Some(colors) = self.active_colors() {
            let base_bg = if selected {
                colors.primary
            } else if has_session {
                Color::from_rgba(
                    colors.secondary.r,
                    colors.secondary.g,
                    colors.secondary.b,
                    0.32,
                )
            } else {
                Color::from_rgba(
                    colors.surface_variant.r,
                    colors.surface_variant.g,
                    colors.surface_variant.b,
                    0.85,
                )
            };
            let text_color = if selected {
                colors.on_primary
            } else {
                colors.on_surface
            };
            btn = btn.style(move |_, status| {
                let mut style = iced::widget::button::Style {
                    background: Some(iced::Background::Color(base_bg)),
                    text_color,
                    border: iced::Border {
                        color: colors.outline,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                };
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(colors.hover));
                    style.text_color = colors.on_primary;
                }
                style
            });
        }
        btn
    }

    fn view_settings_page(&self) -> Element<'_, Message> {
        let theme_picker = pick_list(
            ThemePreference::ALL.as_slice(),
            Some(self.theme_preference),
            Message::ThemeChanged,
        );

        let export_path = if self.export_path.is_empty() {
            "No export file selected".to_string()
        } else {
            self.export_path.clone()
        };
        let import_path = if self.import_path.is_empty() {
            "No import file selected".to_string()
        } else {
            self.import_path.clone()
        };

        let page = column![
            text("Settings").size(34),
            row![text("Theme"), theme_picker]
                .spacing(8)
                .align_y(Alignment::Center),
            row![self.normal_button("Export to file", Message::PickExportPath),]
                .spacing(8)
                .align_y(Alignment::Center),
            text(export_path),
            row![self.normal_button("Import from file", Message::PickImportPath),]
                .spacing(8)
                .align_y(Alignment::Center),
            text(import_path),
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        self.page_container(page)
    }
}

pub fn run(repo: SqliteRepository) -> iced::Result {
    SkillTrackApp::run(repo)
}

fn parse_rfc3339_date(raw: &str) -> Option<NaiveDate> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.date_naive())
}

fn shift_month(date: NaiveDate, by: i32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + by;
    while month < 1 {
        year -= 1;
        month += 12;
    }
    while month > 12 {
        year += 1;
        month -= 12;
    }
    let month_u32 = month as u32;
    let day = date.day().min(days_in_month(year, month_u32));
    NaiveDate::from_ymd_opt(year, month_u32, day).unwrap_or(date)
}

fn shift_year(date: NaiveDate, by: i32) -> NaiveDate {
    let year = date.year() + by;
    let day = date.day().min(days_in_month(year, date.month()));
    NaiveDate::from_ymd_opt(year, date.month(), day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    if let Some(next) = next_month {
        (next - chrono::Duration::days(1)).day()
    } else {
        28
    }
}
