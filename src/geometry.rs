//! Pure screen geometry: the usable desktop rectangle, the notification
//! area, and the strip's placement inside them. No Win32 types — fully
//! unit-tested.

/// Rectangle in physical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    pub const fn width(&self) -> i32 {
        self.right - self.left
    }

    pub const fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Place the strip's bottom-right corner inside `work`, stopping short of
/// the notification area when the bar owns a bottom band.
///
/// `work` is the monitor's work area — the desktop minus whatever the
/// taskbar reserves, whichever edge the bar is docked to — so the strip can
/// never cover the bar itself. `tray` is the shell's notification-area
/// container (`TrayNotifyWnd`); when it sits in a bottom band its left edge
/// becomes the strip's right limit, so the quote ends before the clock, the
/// IME indicator and the hidden-icon chevron instead of hanging over them.
/// A vertical bar's tray has no bottom band and is ignored, as is a hidden
/// bar's (the work area then covers the whole screen).
pub fn place(
    work: &Rect,
    monitor: &Rect,
    tray: Option<Rect>,
    win_w: i32,
    win_h: i32,
    margin: i32,
) -> Rect {
    let area = if work.width() > 0 && work.height() > 0 {
        work
    } else {
        // No work area reported: fall back to the whole screen rather
        // than drawing the strip at a negative coordinate.
        monitor
    };
    let right = match tray {
        Some(t) if t.top + 1 >= area.bottom && t.left > area.left => t.left,
        _ => area.right,
    };
    Rect::new(
        (right - margin - win_w).max(area.left + margin),
        area.bottom - margin - win_h,
        win_w,
        win_h,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MON: Rect = Rect {
        left: 0,
        top: 0,
        right: 3840,
        bottom: 1600,
    };
    const MARGIN: i32 = 10;

    /// Bottom-docked bar: the work area ends above it, and the notification
    /// area (chevron, tray icons, clock) starts at 3526.
    const BOTTOM_WORK: Rect = Rect {
        left: 0,
        top: 0,
        right: 3840,
        bottom: 1552,
    };
    const BOTTOM_TRAY: Rect = Rect {
        left: 3526,
        top: 1552,
        right: 3840,
        bottom: 1600,
    };

    #[test]
    fn strip_stops_left_of_the_notification_area() {
        let r = place(&BOTTOM_WORK, &MON, Some(BOTTOM_TRAY), 500, 45, MARGIN);
        assert_eq!(r, Rect::new(3016, 1497, 500, 45));
        assert!(r.right < BOTTOM_TRAY.left, "never hangs over the tray");
        assert!(r.bottom <= BOTTOM_WORK.bottom, "never sits on the bar");
    }

    #[test]
    fn missing_notification_area_falls_back_to_the_work_edge() {
        let r = place(&BOTTOM_WORK, &MON, None, 500, 45, MARGIN);
        assert_eq!(r, Rect::new(3330, 1497, 500, 45));
    }

    #[test]
    fn left_docked_bar_tray_is_ignored() {
        // A vertical bar's tray sits inside the bar, left of the work area.
        let work = Rect {
            left: 48,
            top: 0,
            right: 3840,
            bottom: 1600,
        };
        let tray = Rect {
            left: 0,
            top: 1552,
            right: 48,
            bottom: 1600,
        };
        let r = place(&work, &MON, Some(tray), 500, 45, MARGIN);
        assert_eq!(r, Rect::new(3330, 1545, 500, 45));
        assert!(r.left >= work.left);
    }

    #[test]
    fn top_docked_bar_tray_is_ignored() {
        let mon = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let work = Rect {
            left: 0,
            top: 48,
            right: 1920,
            bottom: 1080,
        };
        let tray = Rect {
            left: 1700,
            top: 0,
            right: 1920,
            bottom: 48,
        };
        let r = place(&work, &mon, Some(tray), 500, 45, MARGIN);
        assert_eq!(r, Rect::new(1410, 1025, 500, 45));
        assert!(r.top >= work.top);
    }

    #[test]
    fn hidden_bar_leaves_the_screen_edge_available() {
        // Auto-hidden bar: the work area covers the whole screen, and the
        // tray is not in a bottom band.
        let mon = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let tray = Rect {
            left: 1600,
            top: 1032,
            right: 1920,
            bottom: 1080,
        };
        let r = place(&mon, &mon, Some(tray), 500, 45, MARGIN);
        assert_eq!(r, Rect::new(1410, 1025, 500, 45));
    }

    #[test]
    fn empty_work_area_falls_back_to_the_monitor() {
        let work = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let r = place(&work, &MON, None, 500, 45, MARGIN);
        assert_eq!(r, Rect::new(3330, 1545, 500, 45));
    }

    #[test]
    fn narrow_desktop_clamps_to_the_left_edge() {
        // A tray so close to the left edge that the strip would overflow:
        // the strip stops at the left margin instead of going negative.
        let work = Rect {
            left: 0,
            top: 0,
            right: 600,
            bottom: 800,
        };
        let mon = Rect {
            left: 0,
            top: 0,
            right: 600,
            bottom: 800,
        };
        let tray = Rect {
            left: 500,
            top: 800,
            right: 600,
            bottom: 848,
        };
        let r = place(&work, &mon, Some(tray), 500, 45, MARGIN);
        assert_eq!(r.left, MARGIN, "clamped to the left margin, not negative");
    }

    #[test]
    fn rect_helpers() {
        let r = Rect::new(5, 6, 7, 8);
        assert_eq!(r.width(), 7);
        assert_eq!(r.height(), 8);
        assert_eq!(r.right, 12);
        assert_eq!(r.bottom, 14);
    }
}
