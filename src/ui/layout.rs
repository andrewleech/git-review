use ratatui::layout::Rect;

/// Layout information for the TUI
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub header: Option<Rect>,
    pub content: Option<Rect>,
    pub log_pane: Option<Rect>,
    pub diff_area: Option<Rect>,
    pub footer: Option<Rect>,
}

/// Calculate responsive layout based on terminal size
pub fn calculate_layout(
    width: u16,
    height: u16,
    log_visible: bool,
    log_pane_ratio: f32,
) -> LayoutInfo {
    let mut layout = LayoutInfo {
        header: None,
        content: None,
        log_pane: None,
        diff_area: None,
        footer: None,
    };

    if height < 3 {
        // Terminal too small
        return layout;
    }

    let mut y = 0;

    // Header (1 line)
    layout.header = Some(Rect::new(0, y, width, 1));
    y += 1;

    // Content area (height - 2)
    let content_height = height.saturating_sub(2);
    if content_height > 0 {
        layout.content = Some(Rect::new(0, y, width, content_height));

        // Split content if log pane is visible
        if log_visible && width > 40 {
            let log_width = ((width as f32) * log_pane_ratio.clamp(0.1, 0.5)) as u16;
            let log_width = log_width.clamp(20, width / 2); // Min 20 cols, max 50%

            layout.log_pane = Some(Rect::new(0, y, log_width, content_height));
            layout.diff_area = Some(Rect::new(log_width, y, width - log_width, content_height));
        }

        y += content_height;
    }

    // Footer (1 line)
    layout.footer = Some(Rect::new(0, y, width, 1));

    layout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_small_terminal() {
        let layout = calculate_layout(80, 24, true, 0.25);

        assert!(layout.header.is_some());
        assert!(layout.content.is_some());
        assert!(layout.footer.is_some());

        let header = layout.header.unwrap();
        assert_eq!(header.height, 1);
        assert_eq!(header.y, 0);

        let footer = layout.footer.unwrap();
        assert_eq!(footer.height, 1);
        assert_eq!(footer.y, 23);
    }

    #[test]
    fn test_layout_with_log_pane() {
        let layout = calculate_layout(80, 24, true, 0.25);

        assert!(layout.log_pane.is_some());
        assert!(layout.diff_area.is_some());

        let log = layout.log_pane.unwrap();
        assert_eq!(log.width, 20); // 25% of 80

        let diff = layout.diff_area.unwrap();
        assert_eq!(diff.width, 60); // 80 - 20
    }

    #[test]
    fn test_layout_without_log_pane() {
        let layout = calculate_layout(80, 24, false, 0.25);

        assert!(layout.log_pane.is_none());
        assert!(layout.diff_area.is_none());
        assert!(layout.content.is_some());

        let content = layout.content.unwrap();
        assert_eq!(content.width, 80);
    }

    #[test]
    fn test_layout_large_terminal() {
        let layout = calculate_layout(200, 50, true, 0.25);

        let log = layout.log_pane.unwrap();
        assert_eq!(log.width, 50); // 25% of 200

        let diff = layout.diff_area.unwrap();
        assert_eq!(diff.width, 150); // 200 - 50
    }
}
