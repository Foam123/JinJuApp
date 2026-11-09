//! Pure screen geometry: the usable desktop rectangle and the strip's
//! placement inside it. No Win32 types — fully unit-tested.

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

/// Place the strip in the bottom-right corner of the usable desktop.
///
/// `work` is the monitor's work area — the desktop minus whatever the
/// taskbar reserves, whichever edge the bar is docked to. Anchoring to the
/// work area, instead of to the taskbar rectangle plus a docked-edge
/// switch, makes covering the bar structurally impossible: a left-docked
/// bar moves `work.left` right, a bottom-docked bar lifts `work.bottom`
/// up, and so on. The strip therefore stays clear of the bar's own content
/// (search box, icons, tray) at every dock position, and the anchor never
/// depends on the shell answering an appbar query correctly.
pub fn place(work: &Rect, monitor: &Rect, win_w: i32, win_h: i32, margin: i32) -> Rect {
    let area = if work.width() > 0 && work.height() > 0 {
        work
    } else {
        // No work area reported: fall back to the whole screen rather
        // than drawing the strip at a negative coordinate.
        monitor
    };
    Rect::new(
        area.right - win_w - margin,
        area.bottom - win_h - margin,
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

    #[test]
    fn bottom_taskbar_strip_clears_the_bar() {
        // 48 px bottom bar: the work area ends above it, and the strip
        // hangs just inside that boundary — never on the bar itself.
        let work = Rect {
            left: 0,
            top: 0,
            right: 3840,
            bottom: 1552,
        };
        let r = place(&work, &MON, 500, 45, 10);
        assert_eq!(r, Rect::new(3330, 1497, 500, 45));
        assert!(
            r.bottom <= work.bottom,
            "strip must not sit over the taskbar band"
        );
        assert!(r.right <= MON.right && r.bottom <= MON.bottom);
    }

    #[test]
    fn left_taskbar_strip_goes_to_the_right_edge() {
        // A left-docked bar shrinks the work area on the left; the strip
        // still anchors to the desktop's bottom-right corner.
        let work = Rect {
            left: 48,
            top: 0,
            right: 3840,
            bottom: 1600,
        };
        let r = place(&work, &MON, 500, 45, 10);
        assert_eq!(r, Rect::new(3330, 1545, 500, 45));
        assert!(r.left >= work.left, "never over the left-docked bar");
    }

    #[test]
    fn right_taskbar_strip_stays_on_the_usable_desktop() {
        let mon = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let work = Rect {
            left: 0,
            top: 0,
            right: 1872,
            bottom: 1080,
        };
        let r = place(&work, &mon, 500, 45, 10);
        assert_eq!(r, Rect::new(1362, 1025, 500, 45));
        assert!(r.right <= work.right);
    }

    #[test]
    fn top_taskbar_strip_sits_below_the_bar() {
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
        let r = place(&work, &mon, 500, 45, 10);
        assert_eq!(r, Rect::new(1410, 1025, 500, 45));
        assert!(r.top >= work.top);
    }

    #[test]
    fn empty_work_area_falls_back_to_the_monitor() {
        let work = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let r = place(&work, &MON, 500, 45, 10);
        assert_eq!(r, Rect::new(3330, 1545, 500, 45));
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
