use super::super::*;

impl TerminalView {
    pub(crate) fn unified_titlebar_tab_shell_hit_test(
        pointer_x: f32,
        pointer_y: f32,
        tab_widths: impl IntoIterator<Item = f32>,
        scroll_offset_x: f32,
    ) -> bool {
        let tab_top = TOP_STRIP_CONTENT_OFFSET_Y + (TABBAR_HEIGHT - TAB_ITEM_HEIGHT);
        let tab_bottom = TOP_STRIP_CONTENT_OFFSET_Y + TABBAR_HEIGHT;
        if pointer_y < tab_top || pointer_y > tab_bottom {
            return false;
        }

        let mut left = TAB_HORIZONTAL_PADDING + scroll_offset_x;
        for width in tab_widths {
            let right = left + width;
            if pointer_x >= left && pointer_x <= right {
                return true;
            }
            left = right + TAB_ITEM_GAP;
        }

        false
    }

    pub(crate) fn unified_titlebar_tab_interactive_hit_test(
        &self,
        x: f32,
        y: f32,
        window: &Window,
    ) -> bool {
        let geometry = self.tab_strip_geometry(window);

        if geometry.contains_tabs_viewport_x(x) {
            let pointer_x = (x - geometry.row_start_x).clamp(0.0, geometry.tabs_viewport_width);
            let scroll_offset_x: f32 = self.tab_strip.scroll_handle.offset().x.into();
            if Self::unified_titlebar_tab_shell_hit_test(
                pointer_x,
                y,
                self.tabs.iter().map(|tab| tab.display_width),
                scroll_offset_x,
            ) {
                return true;
            }
        }

        if !geometry.contains_action_rail_x(x) {
            return false;
        }

        geometry.new_tab_button_contains(x, y)
    }
}
