use niri_ipc::Window;

use crate::config::WindowRule;

fn matches_window(app_id: &Option<String>, title: &Option<String>, rule: &WindowRule) -> bool {
    let app_ok = match (&rule.app_id, app_id) {
        (None, _) => true,
        (Some(re), Some(id)) => re.is_match(id),
        (Some(_), None) => false,
    };

    let title_ok = match (&rule.title, title) {
        (None, _) => true,
        (Some(re), Some(title)) => re.is_match(title),
        (Some(_), None) => false,
    };

    title_ok && app_ok
}

pub fn resolve_window_size(
    rules: &[WindowRule],
    window: &Window,
    default_w: i32,
    default_h: i32,
) -> (i32, i32) {
    for rule in rules {
        if matches_window(&window.app_id, &window.title, rule) {
            return (
                rule.width.unwrap_or(default_w),
                rule.height.unwrap_or(default_h),
            );
        }
    }
    (default_w, default_h)
}

pub fn resolve_rule_peek(rules: &[WindowRule], window: &Window, default_peek: i32) -> i32 {
    for rule in rules {
        if matches_window(&window.app_id, &window.title, rule) {
            return rule.peek.unwrap_or(default_peek);
        }
    }
    default_peek
}

pub fn resolve_rule_focus_peek(
    rules: &[WindowRule],
    window: &Window,
    default_focus_peek: i32,
) -> i32 {
    for rule in rules {
        if matches_window(&window.app_id, &window.title, rule) {
            return rule.focus_peek.unwrap_or(default_focus_peek);
        }
    }
    default_focus_peek
}

pub fn resolve_auto_add(rules: &[WindowRule], window: &Window) -> bool {
    for rule in rules {
        if matches_window(&window.app_id, &window.title, rule) {
            return rule.auto_add;
        }
    }
    false
}
