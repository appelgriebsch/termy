use super::super::*;
use super::chrome;
use super::layout::TabStripGeometry;
use super::state::{TabDropMarkerSide, TabStripOrientation, TabStripOverflowState};
use gpui::{Hsla, TextRun};

#[derive(Clone, Copy)]
pub(super) struct TabStripPalette {
    pub(super) tab_stroke_color: gpui::Rgba,
    pub(super) inactive_tab_bg: gpui::Rgba,
    pub(super) active_tab_bg: gpui::Rgba,
    pub(super) hovered_tab_bg: gpui::Rgba,
    pub(super) active_tab_text: gpui::Rgba,
    pub(super) inactive_tab_text: gpui::Rgba,
    pub(super) close_button_bg: gpui::Rgba,
    pub(super) close_button_border: gpui::Rgba,
    pub(super) close_button_hover_bg: gpui::Rgba,
    pub(super) close_button_hover_border: gpui::Rgba,
    pub(super) close_button_hover_text: gpui::Rgba,
    pub(super) switch_hint_bg: gpui::Rgba,
    pub(super) switch_hint_border: gpui::Rgba,
    pub(super) switch_hint_text: gpui::Rgba,
    pub(super) tab_drop_marker_color: gpui::Rgba,
    pub(super) tabbar_new_tab_bg: gpui::Rgba,
    pub(super) tabbar_new_tab_hover_bg: gpui::Rgba,
    pub(super) tabbar_new_tab_border: gpui::Rgba,
    pub(super) tabbar_new_tab_hover_border: gpui::Rgba,
    pub(super) tabbar_new_tab_text: gpui::Rgba,
    pub(super) tabbar_new_tab_hover_text: gpui::Rgba,
}

pub(super) struct TabStripRenderState {
    pub(super) geometry: TabStripGeometry,
    pub(super) content_width: f32,
    pub(super) overflow_state: TabStripOverflowState,
    pub(super) chrome_layout: chrome::TabChromeLayout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DividerCollisionState {
    pub(super) left: bool,
    pub(super) right: bool,
}

pub(super) struct TabItemRenderInput {
    pub(super) orientation: TabStripOrientation,
    pub(super) index: usize,
    pub(super) tab_primary_extent: f32,
    pub(super) tab_cross_extent: f32,
    pub(super) tab_strokes: TabItemStrokeRects,
    pub(super) label: String,
    pub(super) switch_hint_label: Option<String>,
    pub(super) is_active: bool,
    pub(super) is_hovered: bool,
    pub(super) is_renaming: bool,
    pub(super) show_tab_close: bool,
    pub(super) close_slot_width: f32,
    pub(super) text_padding_x: f32,
    pub(super) label_centered: bool,
    pub(super) trailing_divider_cover: Option<gpui::Rgba>,
    pub(super) drop_marker_side: Option<TabDropMarkerSide>,
    /// 0.0..=1.0 progress for new-tab open animation, None when not animating
    pub(super) open_anim_progress: Option<f32>,
}

#[derive(Clone, Copy)]
pub(super) struct TabItemStrokeRects {
    pub(super) top: Option<chrome::StrokeRect>,
    pub(super) bottom: Option<chrome::StrokeRect>,
    pub(super) left: Option<chrome::StrokeRect>,
    pub(super) right: Option<chrome::StrokeRect>,
}

#[derive(Clone, Copy)]
pub(super) enum TabStripControlAction {
    NewTab,
    ToggleVerticalSidebar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_chip_fits_within_close_slot() {
        assert!(TAB_CLOSE_CHIP_WIDTH < TAB_CLOSE_SLOT_WIDTH);
        assert!(TAB_CLOSE_CHIP_HEIGHT < TAB_CLOSE_HITBOX);
    }

    #[test]
    fn tab_strip_chrome_visible_matches_auto_hide_policy() {
        assert!(!TerminalView::tab_strip_chrome_visible(true, 1));
        assert!(!TerminalView::tab_strip_chrome_visible(true, 0));
        assert!(TerminalView::tab_strip_chrome_visible(false, 1));
        assert!(TerminalView::tab_strip_chrome_visible(true, 2));
    }
}

impl TerminalView {
    pub(super) fn edge_divider_collision_state(
        layout: &chrome::TabChromeLayout,
        scroll_offset_x: f32,
        tabs_viewport_width: f32,
    ) -> DividerCollisionState {
        let left_divider_start_col = 0_i32;
        let left_divider_end_col = (TAB_STROKE_THICKNESS.ceil() as i32).max(1);
        let right_divider_x = (tabs_viewport_width - TAB_STROKE_THICKNESS).max(0.0);
        let right_divider_start_col = right_divider_x.floor() as i32;
        let right_divider_end_col = ((right_divider_x + TAB_STROKE_THICKNESS).ceil() as i32)
            .max(right_divider_start_col + 1);

        let mut collisions = DividerCollisionState::default();

        for stroke in &layout.boundary_strokes {
            let boundary_left = stroke.x + scroll_offset_x;
            let boundary_start_col = boundary_left.floor() as i32;
            let boundary_end_col =
                ((boundary_left + TAB_STROKE_THICKNESS).ceil() as i32).max(boundary_start_col + 1);

            if boundary_start_col < left_divider_end_col
                && boundary_end_col > left_divider_start_col
            {
                collisions.left = true;
            }
            if boundary_start_col < right_divider_end_col
                && boundary_end_col > right_divider_start_col
            {
                collisions.right = true;
            }

            if collisions.left && collisions.right {
                break;
            }
        }

        collisions
    }

    pub(super) fn measure_text_width(
        &mut self,
        window: &Window,
        font_family: &SharedString,
        font_family_key: &str,
        text: &str,
        font_size_px: f32,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        if !font_size_px.is_finite() || font_size_px <= 0.0 {
            return 0.0;
        }
        let font_size_bits = font_size_px.to_bits();
        if let Some(width) =
            self.tab_strip
                .title_width_cache
                .get(text, font_family_key, font_size_bits)
        {
            return width;
        }

        let run = TextRun {
            len: text.len(),
            font: Font {
                family: font_family.clone(),
                weight: FontWeight::NORMAL,
                ..Default::default()
            },
            color: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 1.0,
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let shaped = window.text_system().shape_line(
            text.to_string().into(),
            px(font_size_px),
            &[run],
            None,
        );
        let width: f32 = shaped.x_for_index(text.len()).into();
        let width = width.max(0.0);
        self.tab_strip
            .title_width_cache
            .insert(text, font_family_key, font_size_bits, width);
        width
    }

    pub(super) fn measure_tab_title_width(
        &mut self,
        window: &Window,
        font_family: &SharedString,
        font_family_key: &str,
        title: &str,
    ) -> f32 {
        self.measure_text_width(window, font_family, font_family_key, title, 12.0)
    }

    pub(super) fn measure_tab_title_widths(
        &mut self,
        window: &Window,
        font_family: &SharedString,
        font_family_key: &str,
    ) -> Vec<f32> {
        let mut widths = Vec::with_capacity(self.tabs.len());
        for index in 0..self.tabs.len() {
            let title = self.tabs[index].title.clone();
            widths.push(self.measure_tab_title_width(window, font_family, font_family_key, &title));
        }
        widths
    }

    pub(super) fn termy_branding_reserved_width(
        &mut self,
        window: &Window,
        font_family: &SharedString,
        font_family_key: &str,
    ) -> f32 {
        if !cfg!(target_os = "macos") || !self.show_termy_in_titlebar {
            return 0.0;
        }

        let text_width = self.measure_text_width(
            window,
            font_family,
            font_family_key,
            TOP_STRIP_TERMY_BRANDING_TEXT,
            TOP_STRIP_TERMY_BRANDING_FONT_SIZE,
        );
        if text_width <= f32::EPSILON {
            return 0.0;
        }

        text_width + (TOP_STRIP_TERMY_BRANDING_SIDE_PADDING * 2.0)
    }

    pub(super) fn resolve_tab_strip_palette(
        &self,
        colors: &TerminalColors,
        tabbar_bg: gpui::Rgba,
    ) -> TabStripPalette {
        let tab_stroke_color = chrome::resolve_tab_stroke_color(
            tabbar_bg,
            colors.foreground,
            TAB_STROKE_FOREGROUND_MIX,
        );
        let mut inactive_tab_bg = colors.foreground;
        inactive_tab_bg.a = self.scaled_chrome_alpha(0.10);
        let mut active_tab_bg = tabbar_bg;
        active_tab_bg.a = 0.0;
        let mut hovered_tab_bg = colors.foreground;
        hovered_tab_bg.a = self.scaled_chrome_alpha(0.13);
        let mut active_tab_text = colors.foreground;
        active_tab_text.a = 0.95;
        let mut inactive_tab_text = colors.foreground;
        inactive_tab_text.a = 0.7;
        let mut close_button_bg = colors.foreground;
        close_button_bg.a = self.scaled_chrome_alpha(0.07);
        let mut close_button_border = colors.foreground;
        close_button_border.a = self.scaled_chrome_alpha(0.14);
        let mut close_button_hover_bg = colors.foreground;
        close_button_hover_bg.a = self.scaled_chrome_alpha(0.16);
        let mut close_button_hover_border = colors.cursor;
        close_button_hover_border.a = self.scaled_chrome_alpha(0.4);
        let mut close_button_hover_text = colors.foreground;
        close_button_hover_text.a = 0.98;
        let now = Instant::now();
        let hint_progress = self.tab_switch_hint_progress(now);
        let mut switch_hint_bg = colors.cursor;
        switch_hint_bg.a = self.scaled_chrome_alpha(0.18 * hint_progress);
        let mut switch_hint_border = colors.cursor;
        switch_hint_border.a = self.scaled_chrome_alpha(0.52 * hint_progress);
        let mut switch_hint_text = colors.foreground;
        switch_hint_text.a = (0.99 * hint_progress).clamp(0.0, 1.0);
        let mut tab_drop_marker_color = colors.cursor;
        tab_drop_marker_color.a = self.scaled_chrome_alpha(0.95);
        let mut tabbar_new_tab_bg = colors.foreground;
        tabbar_new_tab_bg.a = self.scaled_chrome_alpha(0.11);
        let mut tabbar_new_tab_hover_bg = colors.foreground;
        tabbar_new_tab_hover_bg.a = self.scaled_chrome_alpha(0.2);
        let mut tabbar_new_tab_border = colors.foreground;
        tabbar_new_tab_border.a = self.scaled_chrome_alpha(0.24);
        let mut tabbar_new_tab_hover_border = colors.cursor;
        tabbar_new_tab_hover_border.a = self.scaled_chrome_alpha(0.76);
        let mut tabbar_new_tab_text = colors.foreground;
        tabbar_new_tab_text.a = 0.9;
        let mut tabbar_new_tab_hover_text = colors.cursor;
        tabbar_new_tab_hover_text.a = 0.98;

        TabStripPalette {
            tab_stroke_color,
            inactive_tab_bg,
            active_tab_bg,
            hovered_tab_bg,
            active_tab_text,
            inactive_tab_text,
            close_button_bg,
            close_button_border,
            close_button_hover_bg,
            close_button_hover_border,
            close_button_hover_text,
            switch_hint_bg,
            switch_hint_border,
            switch_hint_text,
            tab_drop_marker_color,
            tabbar_new_tab_bg,
            tabbar_new_tab_hover_bg,
            tabbar_new_tab_border,
            tabbar_new_tab_hover_border,
            tabbar_new_tab_text,
            tabbar_new_tab_hover_text,
        }
    }

    pub(super) fn compact_vertical_tab_label(index: usize, title: &str) -> String {
        if index < 9 {
            return (index + 1).to_string();
        }

        title.chars().next().unwrap_or('•').to_string()
    }

    pub(crate) fn tab_strip_chrome_visible(auto_hide_tabbar: bool, tab_count: usize) -> bool {
        !auto_hide_tabbar || tab_count > 1
    }

    pub(crate) fn should_render_tab_strip_chrome(&self) -> bool {
        Self::tab_strip_chrome_visible(self.auto_hide_tabbar, self.tabs.len())
    }

    pub(super) fn build_tab_strip_render_state(
        &mut self,
        window: &Window,
        left_inset_width: f32,
    ) -> TabStripRenderState {
        let viewport_width: f32 = window.viewport_size().width.into();
        let layout =
            Self::tab_strip_layout_for_viewport_with_left_inset(viewport_width, left_inset_width);
        self.set_tab_strip_layout_snapshot(layout);

        let mut geometry = layout.geometry;
        geometry.tabs_viewport_width += geometry.action_rail_width;
        geometry.gutter_start_x += geometry.action_rail_width;
        geometry.action_rail_start_x += geometry.action_rail_width;
        geometry.action_rail_width = 0.0;
        let tab_strip_viewport_width = geometry.tabs_viewport_width;
        let widths_changed =
            self.sync_tab_display_widths_for_viewport_if_needed(tab_strip_viewport_width);
        if widths_changed {
            // Width updates can move the active tab offscreen (especially after
            // tmux snapshot/title sync). Snap once here to keep parity with
            // non-tmux active-tab visibility without overriding manual scrolling.
            self.scroll_active_tab_into_view(TabStripOrientation::Horizontal);
        }
        let content_width = self
            .tab_strip_fixed_content_width()
            .max(tab_strip_viewport_width);
        let overflow_state = self.tab_strip_overflow_state();
        let active_tab_index = (self.active_tab < self.tabs.len()).then_some(self.active_tab);
        let chrome_layout = chrome::compute_tab_chrome_layout(
            self.tabs.iter().map(|tab| tab.display_width),
            chrome::TabChromeInput {
                active_index: active_tab_index,
                tabbar_height: TABBAR_HEIGHT,
                tab_item_height: TAB_ITEM_HEIGHT,
                horizontal_padding: TAB_HORIZONTAL_PADDING,
                tab_item_gap: TAB_ITEM_GAP,
            },
        );
        debug_assert!(chrome_layout.tab_strokes.len() == self.tabs.len());

        TabStripRenderState {
            geometry,
            content_width,
            overflow_state,
            chrome_layout,
        }
    }

    pub(super) fn render_tab_stroke(stroke: chrome::StrokeRect, color: gpui::Rgba) -> AnyElement {
        div()
            .absolute()
            .left(px(stroke.x))
            .top(px(stroke.y))
            .w(px(stroke.w))
            .h(px(stroke.h))
            .bg(color)
            .into_any_element()
    }

    pub(super) fn render_baseline_segments(
        layout: &chrome::TabChromeLayout,
        tab_stroke_color: gpui::Rgba,
    ) -> Vec<AnyElement> {
        let mut elements = Vec::with_capacity(layout.baseline_strokes.len() + 1);
        for segment in &layout.baseline_strokes {
            elements.push(Self::render_tab_stroke(*segment, tab_stroke_color));
        }
        elements.push(
            div()
                .id("tabs-baseline-tail-filler")
                .flex_1()
                .min_w(px(0.0))
                .h(px(TABBAR_HEIGHT))
                .relative()
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(px(layout.baseline_y))
                        .h(px(TAB_STROKE_THICKNESS))
                        .bg(tab_stroke_color),
                )
                .into_any_element(),
        );
        elements
    }

    pub(super) fn render_stroke_segments(
        strokes: &[chrome::StrokeRect],
        tab_stroke_color: gpui::Rgba,
    ) -> Vec<AnyElement> {
        strokes
            .iter()
            .copied()
            .map(|segment| Self::render_tab_stroke(segment, tab_stroke_color))
            .collect()
    }

    pub(super) fn perform_tab_strip_control_action(
        &mut self,
        action: TabStripControlAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            TabStripControlAction::NewTab => {
                self.disarm_titlebar_window_move();
                self.add_tab(cx);
            }
            TabStripControlAction::ToggleVerticalSidebar => {
                if let Err(error) = self.set_vertical_tabs_minimized(!self.vertical_tabs_minimized) {
                    termy_toast::error(error);
                } else {
                    cx.notify();
                }
            }
        }
    }

    fn render_tab_accessory(
        &self,
        input: &TabItemRenderInput,
        palette: &TabStripPalette,
        close_text_color: gpui::Rgba,
        hover_tab_index: usize,
        close_tab_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(label) = input.switch_hint_label.as_ref() {
            let mut accessory = div()
                .flex_none()
                .w(px(input.close_slot_width))
                .h(px(TAB_CLOSE_HITBOX))
                .flex()
                .items_center()
                .justify_center()
                .bg(palette.switch_hint_bg)
                .text_color(palette.switch_hint_text)
                .text_size(px(TAB_SWITCH_HINT_TEXT_SIZE))
                .font_weight(FontWeight::MEDIUM);

            if input.orientation == TabStripOrientation::Horizontal {
                accessory = accessory.border_l_1().border_color(palette.switch_hint_border);
            } else {
                accessory = accessory.border_1().border_color(palette.switch_hint_border);
            }

            return accessory.child(label.clone()).into_any_element();
        }

        div()
            .flex_none()
            .w(px(input.close_slot_width))
            .h(px(TAB_CLOSE_HITBOX))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(TAB_CLOSE_CHIP_WIDTH.min(input.close_slot_width)))
                    .h(px(TAB_CLOSE_CHIP_HEIGHT.min(TAB_CLOSE_HITBOX)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(TAB_CLOSE_CHIP_RADIUS))
                    .bg(palette.close_button_bg)
                    .border_1()
                    .border_color(palette.close_button_border)
                    .text_color(close_text_color)
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                            let is_active = close_tab_index == this.active_tab;
                            if Self::tab_shows_close(
                                this.tab_close_visibility,
                                is_active,
                                this.tab_strip.hovered_tab,
                                this.tab_strip.hovered_tab_close,
                                close_tab_index,
                            ) {
                                this.request_tab_close_by_index(close_tab_index, window, cx);
                                cx.stop_propagation();
                            }
                        }),
                    )
                    .on_mouse_move(
                        cx.listener(move |this, _event: &MouseMoveEvent, _window, cx| {
                            this.on_tab_close_mouse_move(hover_tab_index, cx);
                            cx.stop_propagation();
                        }),
                    )
                    .hover(move |style| {
                        style
                            .bg(palette.close_button_hover_bg)
                            .border_color(palette.close_button_hover_border)
                            .text_color(palette.close_button_hover_text)
                    })
                    .cursor_pointer()
                    .child(div().mt(px(-1.0)).child("×")),
            )
            .into_any_element()
    }

    pub(super) fn render_tab_item(
        &mut self,
        input: TabItemRenderInput,
        font_family: &SharedString,
        colors: &TerminalColors,
        palette: &TabStripPalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let orientation = input.orientation;
        let switch_tab_index = input.index;
        let hover_tab_index = input.index;
        let close_tab_index = input.index;

        let anim = input.open_anim_progress.unwrap_or(1.0);

        let mut rename_text_color = if input.is_active {
            palette.active_tab_text
        } else {
            palette.inactive_tab_text
        };
        rename_text_color.a *= anim;
        let mut rename_selection_color = colors.cursor;
        rename_selection_color.a = if input.is_active { 0.34 } else { 0.24 };
        rename_selection_color.a *= anim;

        let mut tab_bg = if input.is_active {
            palette.active_tab_bg
        } else if input.is_hovered {
            palette.hovered_tab_bg
        } else {
            palette.inactive_tab_bg
        };
        tab_bg.a *= anim;

        let mut close_text_color = if input.is_active {
            palette.active_tab_text
        } else {
            palette.inactive_tab_text
        };
        close_text_color.a *= anim;
        if !input.show_tab_close {
            close_text_color.a = 0.0;
        }

        let accessory_slot = self.render_tab_accessory(
            &input,
            palette,
            close_text_color,
            hover_tab_index,
            close_tab_index,
            cx,
        );

        let justify_label_center = input.label_centered;
        let trailing_divider_cover = input.trailing_divider_cover;
        let mut tab_shell = div()
            .flex_none()
            .relative()
            .overflow_hidden()
            .bg(tab_bg)
            .w(px(input.tab_primary_extent))
            .h(px(input.tab_cross_extent))
            .px(px(input.text_padding_x))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.on_tab_mouse_down(orientation, switch_tab_index, event.click_count, cx);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, event: &MouseMoveEvent, window, cx| {
                    this.on_tab_mouse_move(orientation, hover_tab_index, event, window, cx);
                    cx.stop_propagation();
                }),
            );

        for stroke in [
            input.tab_strokes.top,
            input.tab_strokes.bottom,
            input.tab_strokes.left,
            input.tab_strokes.right,
        ]
        .into_iter()
        .flatten()
        {
            tab_shell = tab_shell.child(Self::render_tab_stroke(stroke, palette.tab_stroke_color));
        }

        let drop_marker = input.drop_marker_side.map(|side| match orientation {
            TabStripOrientation::Horizontal => {
                let marker_x = match side {
                    TabDropMarkerSide::Leading => 0.0,
                    TabDropMarkerSide::Trailing => {
                        input.tab_primary_extent - TAB_DROP_MARKER_WIDTH
                    }
                }
                .max(0.0);
                let marker_height =
                    (input.tab_cross_extent - (TAB_DROP_MARKER_INSET_Y * 2.0)).max(0.0);

                div()
                    .absolute()
                    .left(px(marker_x))
                    .top(px(TAB_DROP_MARKER_INSET_Y))
                    .w(px(TAB_DROP_MARKER_WIDTH))
                    .h(px(marker_height))
                    .bg(palette.tab_drop_marker_color)
            }
            TabStripOrientation::Vertical => {
                let marker_y = match side {
                    TabDropMarkerSide::Leading => 0.0,
                    TabDropMarkerSide::Trailing => {
                        input.tab_cross_extent - TAB_DROP_MARKER_WIDTH
                    }
                }
                .max(0.0);

                div()
                    .absolute()
                    .left(px(TAB_DROP_MARKER_INSET_Y))
                    .top(px(marker_y))
                    .w(px((input.tab_primary_extent - (TAB_DROP_MARKER_INSET_Y * 2.0)).max(0.0)))
                    .h(px(TAB_DROP_MARKER_WIDTH))
                    .bg(palette.tab_drop_marker_color)
            }
        });

        tab_shell
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .relative()
                    .child(if input.is_renaming {
                        self.render_inline_input_layer(
                            Font {
                                family: font_family.clone(),
                                weight: FontWeight::NORMAL,
                                ..Default::default()
                            },
                            px(12.0),
                            rename_text_color.into(),
                            rename_selection_color.into(),
                            InlineInputAlignment::Left,
                            cx,
                        )
                    } else {
                        let mut title_text = div()
                            .size_full()
                            .flex()
                            .items_center()
                            .overflow_x_hidden()
                            .whitespace_nowrap()
                            .font_family(font_family.clone())
                            .text_color(rename_text_color)
                            .text_size(px(12.0))
                            .text_ellipsis();
                        if justify_label_center {
                            title_text = title_text.justify_center();
                        }
                        title_text.child(input.label).into_any_element()
                    }),
            )
            .children((input.close_slot_width > 0.0).then_some(accessory_slot))
            .children(trailing_divider_cover.map(|cover_color| {
                div()
                    .absolute()
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .w(px(TAB_STROKE_THICKNESS))
                    .bg(cover_color)
            }))
            .children(drop_marker)
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_tab_strip_control_button(
        &self,
        id: &'static str,
        icon: &'static str,
        action: TabStripControlAction,
        bg: gpui::Rgba,
        hover_bg: gpui::Rgba,
        border: gpui::Rgba,
        hover_border: gpui::Rgba,
        text: gpui::Rgba,
        hover_text: gpui::Rgba,
        button_size: f32,
        icon_size: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if button_size <= 0.0 {
            return div().id(id).w(px(0.0)).h(px(0.0)).into_any_element();
        }

        let corner_radius = TABBAR_NEW_TAB_BUTTON_RADIUS.min(button_size * 0.5);
        let icon_size = icon_size.min(button_size);

        div()
            .id(id)
            .w(px(button_size))
            .h(px(button_size))
            .rounded(px(corner_radius))
            .bg(bg)
            .border_1()
            .border_color(border)
            .text_color(text)
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                    this.perform_tab_strip_control_action(action, cx);
                    cx.stop_propagation();
                }),
            )
            .hover(move |style| {
                style
                    .bg(hover_bg)
                    .border_color(hover_border)
                    .text_color(hover_text)
            })
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(icon_size))
                    .font_weight(FontWeight::MEDIUM)
                    .mt(px(TABBAR_NEW_TAB_ICON_BASELINE_NUDGE_Y))
                    .child(icon),
            )
            .into_any_element()
    }
}
