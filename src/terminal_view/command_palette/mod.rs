use super::*;
use gpui::point;
use state::{
    command_palette_max_scroll_for_count, command_palette_next_scroll_y,
    command_palette_target_scroll_y, filter_command_palette_items_by_query,
    ordered_theme_ids_for_palette,
};

mod render;
mod state;
mod style;

pub(super) use state::{
    CommandPaletteItem, CommandPaletteItemKind, CommandPaletteMode, CommandPaletteState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandPaletteEscapeAction {
    ClosePalette,
    BackToCommands,
}

impl TerminalView {
    pub(super) fn is_command_palette_open(&self) -> bool {
        self.command_palette.open
    }

    fn command_palette_shortcut(&self, action: CommandAction, window: &Window) -> Option<String> {
        if !self.command_palette.show_keybinds {
            return None;
        }

        action.keybinding_label(window, &self.focus_handle)
    }

    pub(super) fn set_command_palette_mode(
        &mut self,
        mode: CommandPaletteMode,
        animate_selection: bool,
        cx: &mut Context<Self>,
    ) {
        self.command_palette.mode = mode;
        self.command_palette.reset();
        self.inline_input_selecting = false;
        self.refresh_command_palette_matches(animate_selection, cx);
        self.reset_cursor_blink_phase();

        cx.notify();
    }

    pub(super) fn open_command_palette(&mut self, cx: &mut Context<Self>) {
        self.command_palette.open = true;
        self.set_command_palette_mode(CommandPaletteMode::Commands, false, cx);
    }

    pub(super) fn close_command_palette(&mut self, cx: &mut Context<Self>) {
        if !self.command_palette.open {
            return;
        }

        self.command_palette.open = false;
        self.command_palette.mode = CommandPaletteMode::Commands;
        self.command_palette.reset();
        self.inline_input_selecting = false;
        cx.notify();
    }

    fn command_palette_items(&self) -> Vec<CommandPaletteItem> {
        match self.command_palette.mode {
            CommandPaletteMode::Commands => CommandAction::palette_entries()
                .into_iter()
                .map(|entry| CommandPaletteItem::command(entry.title, entry.keywords, entry.action))
                .collect(),
            CommandPaletteMode::Themes => self.command_palette_theme_items(),
        }
    }

    fn command_palette_theme_items(&self) -> Vec<CommandPaletteItem> {
        let theme_ids: Vec<String> = termy_themes::available_theme_ids()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();

        ordered_theme_ids_for_palette(theme_ids, &self.theme_id)
            .into_iter()
            .map(|theme| {
                let is_active = theme == self.theme_id;
                CommandPaletteItem::theme(theme, is_active)
            })
            .collect()
    }

    pub(super) fn filtered_command_palette_items(&self) -> &[CommandPaletteItem] {
        &self.command_palette.filtered_items
    }

    pub(super) fn refresh_command_palette_matches(
        &mut self,
        animate_selection: bool,
        cx: &mut Context<Self>,
    ) {
        self.command_palette.filtered_items = filter_command_palette_items_by_query(
            self.command_palette_items(),
            self.command_palette_query(),
        );
        self.command_palette.clamp_selection();

        if self.command_palette.filtered_items.is_empty() {
            self.command_palette.reset_scroll_animation_state();
            return;
        }

        if animate_selection {
            self.animate_command_palette_to_selected(self.command_palette.filtered_items.len(), cx);
        }
    }

    pub(super) fn animate_command_palette_to_selected(
        &mut self,
        item_count: usize,
        cx: &mut Context<Self>,
    ) {
        if item_count == 0 {
            self.command_palette.reset_scroll_animation_state();
            return;
        }

        let max_scroll = command_palette_max_scroll_for_count(item_count);
        self.command_palette.scroll_max_y = max_scroll;

        let scroll_handle = self.command_palette.base_scroll_handle();
        let offset = scroll_handle.offset();
        let current_y = -Into::<f32>::into(offset.y);
        let Some(target_y) = command_palette_target_scroll_y(
            current_y,
            self.command_palette.selected,
            item_count,
        ) else {
            self.command_palette.reset_scroll_animation_state();
            return;
        };

        if (target_y - current_y).abs() <= f32::EPSILON {
            self.command_palette.scroll_target_y = None;
            self.command_palette.scroll_animating = false;
            self.command_palette.scroll_last_tick = None;
            return;
        }

        self.command_palette.scroll_target_y = Some(target_y);
        self.start_command_palette_scroll_animation(cx);
    }

    fn start_command_palette_scroll_animation(&mut self, cx: &mut Context<Self>) {
        if self.command_palette.scroll_animating {
            return;
        }
        self.command_palette.scroll_animating = true;
        self.command_palette.scroll_last_tick = Some(Instant::now());

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                smol::Timer::after(Duration::from_millis(16)).await;
                let keep_animating = match cx.update(|cx| {
                    this.update(cx, |view, cx| {
                        let changed = view.tick_command_palette_scroll_animation();
                        if changed {
                            cx.notify();
                        }
                        view.command_palette.scroll_animating
                    })
                }) {
                    Ok(keep_animating) => keep_animating,
                    _ => break,
                };

                if !keep_animating {
                    break;
                }
            }
        })
        .detach();
    }

    fn tick_command_palette_scroll_animation(&mut self) -> bool {
        if !self.command_palette.open {
            self.command_palette.reset_scroll_animation_state();
            return false;
        }

        let Some(target_y) = self.command_palette.scroll_target_y else {
            self.command_palette.scroll_animating = false;
            self.command_palette.scroll_last_tick = None;
            return false;
        };

        let scroll_handle = self.command_palette.base_scroll_handle();
        let offset = scroll_handle.offset();
        let current_y = -Into::<f32>::into(offset.y);
        let max_offset_from_handle: f32 = scroll_handle.max_offset().height.into();
        let max_scroll = max_offset_from_handle.max(self.command_palette.scroll_max_y).max(0.0);
        let now = Instant::now();
        let dt = self
            .command_palette
            .scroll_last_tick
            .map(|last| (now - last).as_secs_f32())
            .unwrap_or(1.0 / 60.0);
        self.command_palette.scroll_last_tick = Some(now);

        let next_y = command_palette_next_scroll_y(current_y, target_y, max_scroll, dt);
        scroll_handle.set_offset(point(offset.x, px(-next_y)));

        if (target_y - next_y).abs() <= 0.5 {
            self.command_palette.scroll_target_y = None;
            self.command_palette.scroll_animating = false;
            self.command_palette.scroll_last_tick = None;
            return true;
        }

        true
    }

    pub(super) fn handle_command_palette_key_down(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match key {
            "escape" => {
                match Self::command_palette_escape_action(self.command_palette.mode) {
                    CommandPaletteEscapeAction::ClosePalette => self.close_command_palette(cx),
                    CommandPaletteEscapeAction::BackToCommands => {
                        self.set_command_palette_mode(CommandPaletteMode::Commands, false, cx);
                    }
                }
                return;
            }
            "enter" => {
                self.execute_command_palette_selection(window, cx);
                return;
            }
            "up" => {
                let len = self.filtered_command_palette_items().len();
                if len > 0 && self.command_palette.selected > 0 {
                    self.command_palette.selected -= 1;
                    self.animate_command_palette_to_selected(len, cx);
                    cx.notify();
                }
                return;
            }
            "down" => {
                let len = self.filtered_command_palette_items().len();
                if len > 0 && self.command_palette.selected + 1 < len {
                    self.command_palette.selected += 1;
                    self.animate_command_palette_to_selected(len, cx);
                    cx.notify();
                }
                return;
            }
            _ => {}
        }
    }

    fn command_palette_escape_action(mode: CommandPaletteMode) -> CommandPaletteEscapeAction {
        match mode {
            CommandPaletteMode::Commands => CommandPaletteEscapeAction::ClosePalette,
            CommandPaletteMode::Themes => CommandPaletteEscapeAction::BackToCommands,
        }
    }

    fn execute_command_palette_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = self.filtered_command_palette_items();
        if items.is_empty() {
            return;
        }

        let index = self.command_palette.selected.min(items.len() - 1);
        let item_kind = items[index].kind.clone();

        self.execute_command_palette_item(item_kind, window, cx);
    }

    fn execute_command_palette_item(
        &mut self,
        item_kind: CommandPaletteItemKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match item_kind {
            CommandPaletteItemKind::Command(action) => {
                self.execute_command_palette_action(action, window, cx)
            }
            CommandPaletteItemKind::Theme(theme_id) => self.select_theme_from_palette(&theme_id, cx),
        }
    }

    fn select_theme_from_palette(&mut self, theme_id: &str, cx: &mut Context<Self>) {
        match self.persist_theme_selection(theme_id, cx) {
            Ok(true) => {
                self.close_command_palette(cx);
                termy_toast::success(format!("Theme set to {}", self.theme_id));
                cx.notify();
            }
            Ok(false) => {
                self.close_command_palette(cx);
                termy_toast::info(format!("Theme already set to {}", theme_id));
            }
            Err(error) => {
                termy_toast::error(error);
                cx.notify();
            }
        }
    }

    fn execute_command_palette_action(
        &mut self,
        action: CommandAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let keep_open = action == CommandAction::SwitchTheme;
        if !keep_open {
            self.command_palette.open = false;
            self.command_palette.mode = CommandPaletteMode::Commands;
            self.command_palette.reset();
            self.inline_input_selecting = false;
        }

        self.execute_command_action(action, false, window, cx);

        if keep_open {
            return;
        }

        match action {
            CommandAction::OpenConfig => {
                termy_toast::info("Opened settings file");
                cx.notify();
            }
            CommandAction::NewTab => termy_toast::success("Opened new tab"),
            CommandAction::CloseTab => termy_toast::info("Closed active tab"),
            CommandAction::ZoomIn => termy_toast::info("Zoomed in"),
            CommandAction::ZoomOut => termy_toast::info("Zoomed out"),
            CommandAction::ZoomReset => termy_toast::info("Zoom reset"),
            CommandAction::ImportColors => {}
            CommandAction::Quit
            | CommandAction::SwitchTheme
            | CommandAction::AppInfo
            | CommandAction::NativeSdkExample
            | CommandAction::RestartApp
            | CommandAction::RenameTab
            | CommandAction::MoveTabLeft
            | CommandAction::MoveTabRight
            | CommandAction::SwitchTabLeft
            | CommandAction::SwitchTabRight
            | CommandAction::CheckForUpdates
            | CommandAction::ToggleCommandPalette
            | CommandAction::Copy
            | CommandAction::Paste
            | CommandAction::OpenSearch
            | CommandAction::CloseSearch
            | CommandAction::SearchNext
            | CommandAction::SearchPrevious
            | CommandAction::ToggleSearchCaseSensitive
            | CommandAction::ToggleSearchRegex
            | CommandAction::OpenSettings
            | CommandAction::MinimizeWindow
            | CommandAction::InstallCli => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_action_is_mode_dependent() {
        assert_eq!(
            TerminalView::command_palette_escape_action(CommandPaletteMode::Commands),
            CommandPaletteEscapeAction::ClosePalette
        );
        assert_eq!(
            TerminalView::command_palette_escape_action(CommandPaletteMode::Themes),
            CommandPaletteEscapeAction::BackToCommands
        );
    }
}
