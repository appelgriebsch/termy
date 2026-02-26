use super::*;

impl TerminalView {
    pub(in super::super) fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Focus the terminal on click
        self.focus_handle.focus(window, cx);
        self.reset_cursor_blink_phase();
        let mut changed = false;
        if event.button == MouseButton::Left && self.tab_strip.drag.is_some() {
            self.commit_tab_drag(cx);
        } else if self.reset_tab_drag_state() {
            changed = true;
        }
        if self.clear_tab_hover_state() {
            changed = true;
        }
        if changed {
            cx.notify();
        }

        if event.button != MouseButton::Left {
            return;
        }

        if let Some(hit) = self.terminal_scrollbar_hit_test(event.position, window) {
            self.handle_terminal_scrollbar_mouse_down(hit, window, cx);
            cx.stop_propagation();
            return;
        }

        if Self::is_link_modifier(event.modifiers) {
            if let Some(cell) = self.position_to_cell(event.position, false) {
                if let Some(link) = self.link_at_cell(cell) {
                    if !Self::open_link(&link.target) {
                        termy_toast::error("Failed to open link");
                    }
                    if self.clear_hovered_link() {
                        cx.notify();
                    }
                    return;
                }
            }
        }

        let Some(cell) = self.position_to_cell(event.position, false) else {
            self.clear_selection();
            self.clear_hovered_link();
            cx.notify();
            return;
        };

        if event.click_count >= 3 {
            if self.select_line_at_row(cell.row) {
                self.clear_hovered_link();
                cx.notify();
                return;
            }
        }

        if event.click_count == 2 {
            if self.select_token_at_cell(cell) {
                self.clear_hovered_link();
                cx.notify();
                return;
            }
        }

        self.selection_anchor = Some(cell);
        self.selection_head = Some(cell);
        self.selection_dragging = true;
        self.selection_moved = false;
        self.clear_hovered_link();
        cx.notify();
    }

    pub(in super::super) fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tab_strip.drag.is_some() && !event.dragging() {
            self.commit_tab_drag(cx);
        }

        if self.clear_tab_hover_state() {
            cx.notify();
        }

        if self.terminal_scrollbar_drag.is_some() {
            if event.dragging() {
                self.handle_terminal_scrollbar_drag(event.position, window, cx);
            } else if self.finish_terminal_scrollbar_drag(cx) {
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if !self.selection_dragging || !event.dragging() {
            if Self::is_link_modifier(event.modifiers) {
                let hover_cell = self.position_to_cell(event.position, false);
                if let (Some(cell), Some(current)) = (hover_cell, self.hovered_link.as_ref()) {
                    if current.row == cell.row
                        && (current.start_col..=current.end_col).contains(&cell.col)
                    {
                        return;
                    }
                }

                let next = hover_cell.and_then(|cell| self.link_at_cell(cell));
                if self.hovered_link != next {
                    self.hovered_link = next;
                    cx.notify();
                }
            } else if self.clear_hovered_link() {
                cx.notify();
            }
            return;
        }

        let Some(next_cell) = self.position_to_cell(event.position, true) else {
            return;
        };

        if self.selection_head != Some(next_cell) {
            self.selection_head = Some(next_cell);
            if self.selection_anchor != self.selection_head {
                self.selection_moved = true;
            }
            self.clear_hovered_link();
            cx.notify();
        }
    }

    pub(in super::super) fn handle_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button == MouseButton::Left && self.finish_terminal_scrollbar_drag(cx) {
            cx.stop_propagation();
            cx.notify();
            return;
        }

        if event.button != MouseButton::Left || !self.selection_dragging {
            return;
        }

        if let Some(next_cell) = self.position_to_cell(event.position, true) {
            self.selection_head = Some(next_cell);
            if self.selection_anchor != self.selection_head {
                self.selection_moved = true;
            }
        }

        self.selection_dragging = false;
        if !self.selection_moved {
            self.clear_selection();
        }
        self.clear_hovered_link();
        cx.notify();
    }
}
