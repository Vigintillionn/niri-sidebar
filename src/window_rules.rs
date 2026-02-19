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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::mock_window;
    use regex::Regex;

    #[test]
    fn test_resolve_window_size_defaults() {
        let rules = vec![];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let (w, h) = resolve_window_size(&rules, &window, 100, 200);
        assert_eq!(w, 100);
        assert_eq!(h, 200);
    }

    #[test]
    fn test_resolve_window_size_match_app_id() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("test").unwrap()),
            width: Some(500),
            height: Some(600),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0))); // mock_window has app_id "test"
        let (w, h) = resolve_window_size(&rules, &window, 100, 200);
        assert_eq!(w, 500);
        assert_eq!(h, 600);
    }

    #[test]
    fn test_resolve_window_size_match_title() {
        let rules = vec![WindowRule {
            title: Some(Regex::new("Test Window").unwrap()),
            width: Some(800),
            height: Some(900),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0))); // mock_window has title "Test Window"
        let (w, h) = resolve_window_size(&rules, &window, 100, 200);
        assert_eq!(w, 800);
        assert_eq!(h, 900);
    }

    #[test]
    fn test_resolve_window_size_no_match() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("nomatch").unwrap()),
            width: Some(500),
            height: Some(600),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let (w, h) = resolve_window_size(&rules, &window, 100, 200);
        assert_eq!(w, 100);
        assert_eq!(h, 200);
    }

    #[test]
    fn test_resolve_rule_peek_match() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("test").unwrap()),
            peek: Some(50),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let peek = resolve_rule_peek(&rules, &window, 10);
        assert_eq!(peek, 50);
    }

    #[test]
    fn test_resolve_rule_peek_default() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("nomatch").unwrap()),
            peek: Some(50),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let peek = resolve_rule_peek(&rules, &window, 10);
        assert_eq!(peek, 10);
    }

    #[test]
    fn test_resolve_rule_focus_peek_match() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("test").unwrap()),
            focus_peek: Some(70),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let peek = resolve_rule_focus_peek(&rules, &window, 20);
        assert_eq!(peek, 70);
    }

    #[test]
    fn test_resolve_rule_focus_peek_default() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("nomatch").unwrap()),
            focus_peek: Some(70),
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let peek = resolve_rule_focus_peek(&rules, &window, 20);
        assert_eq!(peek, 20);
    }

    #[test]
    fn test_resolve_auto_add_match() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("test").unwrap()),
            auto_add: true,
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let auto_add = resolve_auto_add(&rules, &window);
        assert!(auto_add);
    }

    #[test]
    fn test_resolve_auto_add_default_false() {
        let rules = vec![WindowRule {
            app_id: Some(Regex::new("nomatch").unwrap()),
            auto_add: true,
            ..Default::default()
        }];
        let window = mock_window(1, false, false, 1, Some((1.0, 2.0)));
        let auto_add = resolve_auto_add(&rules, &window);
        assert!(!auto_add);
    }
}
