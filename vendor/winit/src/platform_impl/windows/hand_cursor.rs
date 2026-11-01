//! GridTool patch (see `PATCHES.md`): real open and closed hands for [`CursorIcon::Grab`] and
//! [`CursorIcon::Grabbing`], in place of the four-way arrow upstream uses for both.

use std::sync::OnceLock;

use cursor_icon::CursorIcon;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, HCURSOR, SM_CXCURSOR};

use crate::cursor::CursorImage;

use super::hand_cursor_art::hand_rgba;
use super::icon::WinCursor;

/// Used if the system does not report its cursor size.
const DEFAULT_SIZE: i32 = 32;

/// The hand for `icon`, if it is one and could be created. Hands are created once, on first
/// use, and last as long as the process.
pub(super) fn hand_cursor(icon: CursorIcon) -> Option<HCURSOR> {
    static OPEN: OnceLock<Option<WinCursor>> = OnceLock::new();
    static CLOSED: OnceLock<Option<WinCursor>> = OnceLock::new();
    let (cache, closed) = match icon {
        CursorIcon::Grab => (&OPEN, false),
        CursorIcon::Grabbing => (&CLOSED, true),
        _ => return None,
    };
    match cache.get_or_init(|| create(closed)) {
        Some(WinCursor::Cursor(cursor)) => Some(cursor.as_raw_handle()),
        _ => None,
    }
}

fn create(closed: bool) -> Option<WinCursor> {
    let reported = unsafe { GetSystemMetrics(SM_CXCURSOR) };
    let size = if reported > 0 { reported } else { DEFAULT_SIZE } as u16;
    let rgba = hand_rgba(closed, u32::from(size));
    // The hot spot is the middle of the hand.
    let image = CursorImage::from_rgba(rgba, size, size, size / 2, size / 2).ok()?;
    WinCursor::new(&image).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_impl::platform::util;
    use windows_sys::Win32::UI::WindowsAndMessaging::{LoadCursorW, IDC_ARROW, IDC_SIZEALL};

    #[test]
    fn grab_and_grabbing_are_their_own_cursors_and_the_rest_are_the_systems() {
        let four_way = unsafe { LoadCursorW(0, IDC_SIZEALL) };
        let arrow = unsafe { LoadCursorW(0, IDC_ARROW) };
        assert_ne!(four_way, 0);

        let open = util::load_cursor(CursorIcon::Grab);
        let closed = util::load_cursor(CursorIcon::Grabbing);

        assert_ne!(open, 0);
        assert_ne!(closed, 0);
        assert_ne!(open, closed);
        assert_ne!(open, four_way);
        assert_ne!(closed, four_way);
        // Created only once.
        assert_eq!(util::load_cursor(CursorIcon::Grab), open);

        assert_eq!(util::load_cursor(CursorIcon::Move), four_way);
        assert_eq!(util::load_cursor(CursorIcon::Default), arrow);
    }
}
