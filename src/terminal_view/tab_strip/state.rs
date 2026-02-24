use gpui::ScrollHandle;

use super::layout::TabStripLayoutSnapshot;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TabDragState {
    pub(crate) source_index: usize,
    pub(crate) drop_slot: Option<usize>,
}

pub(crate) struct TabStripState {
    pub(crate) scroll_handle: ScrollHandle,
    pub(crate) hovered_tab: Option<usize>,
    pub(crate) hovered_tab_close: Option<usize>,
    pub(crate) drag: Option<TabDragState>,
    pub(crate) drag_pointer_x: Option<f32>,
    pub(crate) drag_viewport_width: f32,
    pub(crate) drag_autoscroll_animating: bool,
    pub(crate) layout_revision: u64,
    pub(crate) layout_last_synced_revision: u64,
    pub(crate) layout_last_synced_viewport_width: f32,
    pub(crate) layout_snapshot: Option<TabStripLayoutSnapshot>,
}

impl TabStripState {
    pub(crate) fn new() -> Self {
        Self {
            scroll_handle: ScrollHandle::new(),
            hovered_tab: None,
            hovered_tab_close: None,
            drag: None,
            drag_pointer_x: None,
            drag_viewport_width: 0.0,
            drag_autoscroll_animating: false,
            layout_revision: 0,
            layout_last_synced_revision: 0,
            layout_last_synced_viewport_width: f32::NAN,
            layout_snapshot: None,
        }
    }
}
