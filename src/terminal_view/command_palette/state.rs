use super::super::*;
use crate::config::SHELL_DECIDE_THEME_ID;
use gpui::UniformListScrollHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum CommandPaletteMode {
    Commands,
    Themes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum CommandPaletteItemKind {
    Command(CommandAction),
    Theme(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) struct CommandPaletteItem {
    pub(in super::super) title: String,
    pub(in super::super) keywords: String,
    pub(in super::super) kind: CommandPaletteItemKind,
}

impl CommandPaletteItem {
    pub(in super::super) fn command(title: &str, keywords: &str, action: CommandAction) -> Self {
        Self {
            title: title.to_string(),
            keywords: keywords.to_string(),
            kind: CommandPaletteItemKind::Command(action),
        }
    }

    pub(in super::super) fn theme(theme_id: String, is_active: bool) -> Self {
        let title = if is_active {
            format!("\u{2713} {}", theme_id)
        } else {
            theme_id.clone()
        };
        let keywords = format!("theme palette colors {}", theme_id.replace('-', " "));

        Self {
            title,
            keywords,
            kind: CommandPaletteItemKind::Theme(theme_id),
        }
    }
}

#[derive(Clone, Debug)]
pub(in super::super) struct CommandPaletteState {
    pub(in super::super) open: bool,
    pub(in super::super) mode: CommandPaletteMode,
    pub(in super::super) input: InlineInputState,
    pub(in super::super) filtered_items: Vec<CommandPaletteItem>,
    pub(in super::super) selected: usize,
    pub(in super::super) scroll_handle: UniformListScrollHandle,
    pub(in super::super) scroll_target_y: Option<f32>,
    pub(in super::super) scroll_max_y: f32,
    pub(in super::super) scroll_animating: bool,
    pub(in super::super) scroll_last_tick: Option<Instant>,
    pub(in super::super) show_keybinds: bool,
}

impl CommandPaletteState {
    pub(in super::super) fn new(show_keybinds: bool) -> Self {
        Self {
            open: false,
            mode: CommandPaletteMode::Commands,
            input: InlineInputState::new(String::new()),
            filtered_items: Vec::new(),
            selected: 0,
            scroll_handle: UniformListScrollHandle::new(),
            scroll_target_y: None,
            scroll_max_y: 0.0,
            scroll_animating: false,
            scroll_last_tick: None,
            show_keybinds,
        }
    }

    pub(in super::super) fn base_scroll_handle(&self) -> gpui::ScrollHandle {
        self.scroll_handle.0.borrow().base_handle.clone()
    }

    pub(in super::super) fn reset_scroll_animation_state(&mut self) {
        self.scroll_target_y = None;
        self.scroll_max_y = 0.0;
        self.scroll_animating = false;
        self.scroll_last_tick = None;
    }

    pub(in super::super) fn reset(&mut self) {
        self.input.clear();
        self.filtered_items.clear();
        self.selected = 0;
        self.scroll_handle = UniformListScrollHandle::new();
        self.reset_scroll_animation_state();
    }

    pub(in super::super) fn clamp_selection(&mut self) {
        if self.filtered_items.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_items.len() {
            self.selected = self.filtered_items.len() - 1;
        }
    }
}

pub(in super::super) fn ordered_theme_ids_for_palette(
    mut theme_ids: Vec<String>,
    current_theme: &str,
) -> Vec<String> {
    if !theme_ids.iter().any(|theme| theme == SHELL_DECIDE_THEME_ID) {
        theme_ids.push(SHELL_DECIDE_THEME_ID.to_string());
    }

    if !theme_ids.iter().any(|theme| theme == current_theme) {
        theme_ids.push(current_theme.to_string());
    }

    theme_ids.sort_unstable();
    theme_ids.dedup();

    if let Some(current_index) = theme_ids.iter().position(|theme| theme == current_theme) {
        let current = theme_ids.remove(current_index);
        theme_ids.insert(0, current);
    }

    theme_ids
}

pub(in super::super) fn filter_command_palette_items_by_query(
    items: Vec<CommandPaletteItem>,
    query: &str,
) -> Vec<CommandPaletteItem> {
    let query = query.trim().to_ascii_lowercase();
    let query_terms: Vec<String> = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if query_terms.is_empty() {
        return items;
    }

    let has_title_matches = items
        .iter()
        .any(|item| command_palette_text_matches_terms(&item.title, &query_terms));

    items
        .into_iter()
        .filter(|item| {
            let title_match = command_palette_text_matches_terms(&item.title, &query_terms);
            if has_title_matches {
                title_match
            } else {
                title_match || command_palette_text_matches_terms(&item.keywords, &query_terms)
            }
        })
        .collect()
}

fn command_palette_text_matches_terms(text: &str, query_terms: &[String]) -> bool {
    let searchable = text.to_ascii_lowercase();
    let words: Vec<&str> = searchable
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();

    query_terms
        .iter()
        .all(|term| words.iter().any(|word| word.starts_with(term)))
}

pub(in super::super) fn command_palette_viewport_height() -> f32 {
    COMMAND_PALETTE_MAX_ITEMS as f32 * COMMAND_PALETTE_ROW_HEIGHT
}

pub(in super::super) fn command_palette_max_scroll_for_count(item_count: usize) -> f32 {
    (item_count as f32 * COMMAND_PALETTE_ROW_HEIGHT - command_palette_viewport_height()).max(0.0)
}

pub(in super::super) fn command_palette_target_scroll_y(
    current_y: f32,
    selected_index: usize,
    item_count: usize,
) -> Option<f32> {
    if item_count == 0 {
        return None;
    }

    let viewport_height = command_palette_viewport_height();
    let max_scroll = command_palette_max_scroll_for_count(item_count);
    let row_top = selected_index as f32 * COMMAND_PALETTE_ROW_HEIGHT;
    let row_bottom = row_top + COMMAND_PALETTE_ROW_HEIGHT;

    let target = if row_top < current_y {
        row_top
    } else if row_bottom > current_y + viewport_height {
        row_bottom - viewport_height
    } else {
        current_y
    };

    Some(target.clamp(0.0, max_scroll))
}

pub(in super::super) fn command_palette_next_scroll_y(
    current_y: f32,
    target_y: f32,
    max_scroll: f32,
    dt_seconds: f32,
) -> f32 {
    let target_y = target_y.clamp(0.0, max_scroll);
    let delta = target_y - current_y;
    if delta.abs() <= 0.5 {
        return target_y;
    }

    let dt = dt_seconds.clamp(1.0 / 240.0, 0.05);
    let smoothing = 1.0 - (-18.0 * dt).exp();
    let desired_step = delta * smoothing;
    let max_step = 1800.0 * dt;
    let step = desired_step.clamp(-max_step, max_step);
    let next_y = (current_y + step).clamp(0.0, max_scroll);

    if (target_y - next_y).abs() <= 0.5 {
        target_y
    } else {
        next_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_item(title: &str, keywords: &str, action: CommandAction) -> CommandPaletteItem {
        CommandPaletteItem::command(title, keywords, action)
    }

    #[test]
    fn query_re_prefers_title_matches_over_keywords() {
        let items = vec![
            command_item("Close Tab", "remove tab", CommandAction::CloseTab),
            command_item("Rename Tab", "title name", CommandAction::RenameTab),
            command_item(
                "Restart App",
                "relaunch reopen restart",
                CommandAction::RestartApp,
            ),
            command_item("Reset Zoom", "font default", CommandAction::ZoomReset),
            command_item(
                "Check for Updates",
                "release version updater",
                CommandAction::CheckForUpdates,
            ),
        ];

        let filtered = filter_command_palette_items_by_query(items, "re");
        let actions: Vec<CommandAction> = filtered
            .into_iter()
            .filter_map(|item| match item.kind {
                CommandPaletteItemKind::Command(action) => Some(action),
                CommandPaletteItemKind::Theme(_) => None,
            })
            .collect();

        assert_eq!(
            actions,
            vec![
                CommandAction::RenameTab,
                CommandAction::RestartApp,
                CommandAction::ZoomReset
            ]
        );
    }

    #[test]
    fn query_uses_keywords_when_no_titles_match() {
        let items = vec![
            command_item("Zoom In", "font increase", CommandAction::ZoomIn),
            command_item("Zoom Out", "font decrease", CommandAction::ZoomOut),
            command_item("Reset Zoom", "font default", CommandAction::ZoomReset),
        ];

        let filtered = filter_command_palette_items_by_query(items, "font");
        let actions: Vec<CommandAction> = filtered
            .into_iter()
            .filter_map(|item| match item.kind {
                CommandPaletteItemKind::Command(action) => Some(action),
                CommandPaletteItemKind::Theme(_) => None,
            })
            .collect();

        assert_eq!(
            actions,
            vec![
                CommandAction::ZoomIn,
                CommandAction::ZoomOut,
                CommandAction::ZoomReset
            ]
        );
    }

    #[test]
    fn target_scroll_y_only_moves_when_selection_leaves_viewport() {
        assert_eq!(command_palette_target_scroll_y(0.0, 2, 12), Some(0.0));
        assert_eq!(command_palette_target_scroll_y(0.0, 9, 12), Some(60.0));
        assert_eq!(command_palette_target_scroll_y(90.0, 0, 12), Some(0.0));
        assert_eq!(command_palette_target_scroll_y(0.0, 0, 0), None);
    }

    #[test]
    fn next_scroll_y_is_dt_based_and_respects_bounds() {
        let slow = command_palette_next_scroll_y(0.0, 120.0, 300.0, 1.0 / 240.0);
        let fast = command_palette_next_scroll_y(0.0, 120.0, 300.0, 0.05);
        assert!(fast > slow);
        assert!(fast <= 300.0);

        let snapped = command_palette_next_scroll_y(59.7, 60.0, 300.0, 1.0 / 60.0);
        assert_eq!(snapped, 60.0);

        let clamped = command_palette_next_scroll_y(280.0, 400.0, 300.0, 0.05);
        assert!(clamped <= 300.0);
    }

    #[test]
    fn ordered_theme_ids_pin_current_theme_first() {
        let ordered = ordered_theme_ids_for_palette(
            vec![
                "nord".to_string(),
                "termy".to_string(),
                "dracula".to_string(),
                "nord".to_string(),
            ],
            "termy",
        );

        assert_eq!(
            ordered,
            vec!["termy", "dracula", "nord", SHELL_DECIDE_THEME_ID]
        );

        let ordered_with_missing_current =
            ordered_theme_ids_for_palette(vec!["nord".to_string(), "dracula".to_string()], "tokyo-night");

        assert_eq!(
            ordered_with_missing_current,
            vec!["tokyo-night", "dracula", "nord", SHELL_DECIDE_THEME_ID]
        );
    }

    #[test]
    fn state_reset_clears_transient_fields() {
        let mut state = CommandPaletteState::new(false);
        state.open = true;
        state.mode = CommandPaletteMode::Themes;
        state.input.set_text("theme".to_string());
        state.filtered_items = vec![CommandPaletteItem::command(
            "New Tab",
            "tab",
            CommandAction::NewTab,
        )];
        state.selected = 99;
        state.scroll_target_y = Some(12.0);
        state.scroll_max_y = 40.0;
        state.scroll_animating = true;

        state.reset();

        assert!(state.input.text().is_empty());
        assert!(state.filtered_items.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.scroll_target_y.is_none());
        assert_eq!(state.scroll_max_y, 0.0);
        assert!(!state.scroll_animating);
    }
}
