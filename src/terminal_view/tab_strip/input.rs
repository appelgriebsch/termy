use super::super::*;

impl TerminalView {
    pub(crate) fn tab_strip_scroll_delta_from_pixels(delta_x: f32, delta_y: f32) -> f32 {
        if delta_x.abs() <= f32::EPSILON && delta_y.abs() <= f32::EPSILON {
            return 0.0;
        }

        let dominant_delta = if delta_x.abs() >= delta_y.abs() {
            delta_x
        } else {
            delta_y
        };

        // ScrollHandle offset-space is [-max, 0], while input deltas are content-space.
        // Invert once here so all callers can pass the resulting offset delta directly.
        -dominant_delta
    }

    pub(crate) fn handle_tab_strip_action_rail_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pixel_delta = event
            .delta
            .pixel_delta(px(TAB_STRIP_WHEEL_DELTA_LINE_REFERENCE_PX));
        let delta_x: f32 = pixel_delta.x.into();
        let delta_y: f32 = pixel_delta.y.into();
        let scroll_delta = Self::tab_strip_scroll_delta_from_pixels(delta_x, delta_y);
        if self.scroll_tab_strip_by(scroll_delta) {
            cx.notify();
        }
        cx.stop_propagation();
    }

    fn arm_titlebar_window_move(&mut self) {
        self.titlebar_move_armed = true;
    }

    pub(crate) fn disarm_titlebar_window_move(&mut self) {
        self.titlebar_move_armed = false;
    }

    pub(crate) fn titlebar_move_armed_after_mouse_down(
        interactive_hit: bool,
        click_count: usize,
    ) -> bool {
        !interactive_hit && click_count != 2
    }

    pub(crate) fn titlebar_move_armed_after_mouse_up() -> bool {
        false
    }

    pub(crate) fn handle_unified_titlebar_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }

        let x: f32 = event.position.x.into();
        let y: f32 = event.position.y.into();
        let interactive_hit = self.unified_titlebar_tab_interactive_hit_test(x, y, window);
        let next_move_armed =
            Self::titlebar_move_armed_after_mouse_down(interactive_hit, event.click_count);
        if !next_move_armed {
            self.disarm_titlebar_window_move();
        }
        if !interactive_hit && event.click_count == 2 {
            #[cfg(target_os = "macos")]
            window.titlebar_double_click();
            #[cfg(not(target_os = "macos"))]
            window.zoom_window();
            cx.stop_propagation();
            return;
        }

        if next_move_armed {
            self.arm_titlebar_window_move();
            cx.stop_propagation();
        }
    }

    pub(crate) fn handle_unified_titlebar_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }

        self.titlebar_move_armed = Self::titlebar_move_armed_after_mouse_up();
        cx.stop_propagation();
    }

    pub(crate) fn maybe_start_titlebar_window_move(
        &mut self,
        dragging: bool,
        window: &mut Window,
    ) -> bool {
        if !Self::should_start_titlebar_window_move(
            self.titlebar_move_armed,
            dragging,
            self.tab_strip.drag.is_some(),
        ) {
            return false;
        }

        self.disarm_titlebar_window_move();
        window.start_window_move();
        true
    }

    pub(crate) fn should_start_titlebar_window_move(
        titlebar_move_armed: bool,
        dragging: bool,
        tab_drag_active: bool,
    ) -> bool {
        titlebar_move_armed && dragging && !tab_drag_active
    }

    pub(crate) fn handle_titlebar_tab_strip_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.maybe_start_titlebar_window_move(event.dragging(), window) {
            cx.stop_propagation();
            return;
        }

        let mut changed = false;
        if self.tab_strip.hovered_tab.take().is_some() || self.tab_strip.hovered_tab_close.take().is_some()
        {
            changed = true;
        }
        if event.dragging() {
            let (pointer_x, viewport_width) =
                self.tab_strip_pointer_x_from_window_x(window, event.position.x);
            if !self.update_tab_drag_preview(pointer_x, viewport_width, cx) && changed {
                cx.notify();
            }
            return;
        }
        if self.tab_strip.drag.is_some() {
            self.commit_tab_drag(cx);
        }
        if changed {
            cx.notify();
        }
    }
}
