//! Search: a command palette over every engine and UI command (Window › Search, Control+Shift+S on macOS).

use crate::SoundApp;
use crate::theme::{Tokens, mono};
use egui::{Align2, Key, vec2};

#[derive(Debug, Clone, Default)]
pub struct PaletteState {
    pub query: String,
    pub selected: usize,
}

struct Entry {
    action: EntryAction,
    label: String,
    path: String,
    shortcut: String,
}

#[derive(Debug, Clone)]
enum EntryAction {
    Command { id: String, path: String },
    SelectTrack(u64),
    SelectClip(u64),
    RecallMarker(u32),
    ToggleGroup(u64),
}

fn entries(app: &SoundApp) -> Vec<Entry> {
    let mut v: Vec<Entry> = soundcraft_engine::command_specs()
        .iter()
        .map(|c| Entry {
            action: EntryAction::Command { id: c.id.to_string(), path: c.menu.join(" › ") },
            label: c.label.to_string(),
            path: c.menu.join(" › "),
            shortcut: c.shortcut.unwrap_or("").to_string(),
        })
        .collect();
    for (id, label, path, sc) in crate::menus::UI_COMMANDS {
        let path = path.replace(" > ", " › ");
        v.push(Entry {
            action: EntryAction::Command { id: id.to_string(), path: path.clone() },
            label: label.to_string(),
            path,
            shortcut: sc.unwrap_or("").to_string(),
        });
    }
    let s = app.engine.session();
    for track in &s.tracks {
        v.push(Entry { action: EntryAction::SelectTrack(track.id.0), label: track.name.clone(), path: "Tracks".into(), shortcut: String::new() });
    }
    for clip in s.all_clips() {
        v.push(Entry { action: EntryAction::SelectClip(clip.id.0), label: clip.name.clone(), path: "Clips".into(), shortcut: String::new() });
    }
    for marker in &s.markers {
        v.push(Entry {
            action: EntryAction::RecallMarker(marker.number),
            label: if marker.name.is_empty() { format!("Marker {}", marker.number) } else { marker.name.clone() },
            path: "Memory Locations".into(),
            shortcut: String::new(),
        });
    }
    for group in &s.groups {
        v.push(Entry { action: EntryAction::ToggleGroup(group.id.0), label: group.name.clone(), path: "Groups".into(), shortcut: String::new() });
    }
    v
}

fn run_entry(app: &mut SoundApp, entry: &EntryAction) {
    match entry {
        EntryAction::Command { id, path } => crate::menus::invoke_menu(app, id, &path.replace(" › ", " > ")),
        EntryAction::SelectTrack(id) => {
            let _ = app.run("edit.select", serde_json::json!({"tracks": [id]}));
        }
        EntryAction::SelectClip(id) => {
            let _ = app.run("edit.select", serde_json::json!({"clips": [id]}));
        }
        EntryAction::RecallMarker(number) => {
            let _ = app.run("markers.recall", serde_json::json!({"number": number}));
        }
        EntryAction::ToggleGroup(id) => {
            let _ = app.run("track.group_toggle", serde_json::json!({"group": id}));
        }
    }
}

/// Subsequence match score (higher is better), or None.
fn score(hay: &str, needle: &str) -> Option<i32> {
    if needle.is_empty() {
        return Some(0);
    }
    let h = hay.to_lowercase();
    let n = needle.to_lowercase();
    if let Some(i) = h.find(&n) {
        return Some(1000 - i32::try_from(i).unwrap_or(999));
    }
    let mut it = h.chars();
    let mut gaps = 0;
    for c in n.chars() {
        let mut found = false;
        for hc in it.by_ref() {
            if hc == c {
                found = true;
                break;
            }
            gaps += 1;
        }
        if !found {
            return None;
        }
    }
    Some(500 - gaps)
}

pub fn show(app: &mut SoundApp, ctx: &egui::Context) {
    if !app.ui.show_search {
        return;
    }
    let t = Tokens::current();
    let mut run: Option<EntryAction> = None;
    let mut close = ctx.input(|i| i.key_pressed(Key::Escape));
    egui::Window::new(crate::localization::text("Search"))
        .id(egui::Id::unique("Search"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_TOP, vec2(0.0, 90.0))
        .fixed_size(vec2(560.0, 380.0))
        .show(ctx, |ui| {
            let r = ui.add(
                egui::TextEdit::singleline(&mut app.palette.query)
                    .hint_text(crate::localization::text("Search commands…"))
                    .desired_width(f32::INFINITY)
                    .font(mono(15.0)),
            );
            r.request_focus();
            let q = app.palette.query.trim().to_string();
            let mut hits: Vec<(i32, Entry)> = entries(app)
                .into_iter()
                .filter_map(|e| {
                    let label = crate::localization::text(&e.label);
                    let path = e.path.split(" › ").map(crate::localization::text).collect::<Vec<_>>().join(" › ");
                    score(&format!("{label} {path} {} {}", e.label, e.path), &q).map(|s| (s, e))
                })
                .collect();
            hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.label.cmp(&b.1.label)));
            hits.truncate(14);
            let (down, up, enter) = ctx.input(|i| (i.key_pressed(Key::ArrowDown), i.key_pressed(Key::ArrowUp), i.key_pressed(Key::Enter)));
            if down {
                app.palette.selected = (app.palette.selected + 1).min(hits.len().saturating_sub(1));
            }
            if up {
                app.palette.selected = app.palette.selected.saturating_sub(1);
            }
            app.palette.selected = app.palette.selected.min(hits.len().saturating_sub(1));
            ui.separator();
            for (i, (_, e)) in hits.iter().enumerate() {
                let sel = i == app.palette.selected;
                let resp = ui.horizontal(|ui| {
                    let l = ui.selectable_label(sel, crate::localization::text(&e.label));
                    let path = e.path.split(" › ").map(crate::localization::text).collect::<Vec<_>>().join(" › ");
                    ui.label(egui::RichText::new(path).small().color(t.text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sc = crate::shortcuts::display_shortcut(&e.shortcut, ctx.os().is_mac());
                        ui.label(egui::RichText::new(sc).font(mono(10.0)).color(t.text_dim));
                    });
                    l
                });
                if resp.inner.clicked() {
                    run = Some(e.action.clone());
                }
            }
            if enter && let Some((_, e)) = hits.get(app.palette.selected) {
                run = Some(e.action.clone());
            }
        });
    if let Some(action) = run {
        close = true;
        run_entry(app, &action);
    }
    if close {
        app.ui.show_search = false;
        app.palette = PaletteState::default();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn scoring_prefers_substrings() {
        assert!(super::score("Duplicate", "dup") > super::score("Duplicate", "dpl"));
        assert!(super::score("Bounce Mix", "xyz").is_none());
        assert_eq!(super::score("anything", ""), Some(0));
    }
}
