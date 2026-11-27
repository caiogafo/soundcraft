use super::{Align2, ClipId, Rect, Sense, SoundApp, SourceId, Tokens, Ui, bold, json, panel_header, pos2, regular, rgb, vec2, with_pending_offset};

/// One row of the clip list: a whole file, or a clip on a track (positions in the session).
#[derive(Clone, Copy)]
enum ClipRow {
    Source(usize),
    Clip { track: usize, clip: usize },
}

/// The clip list's rows for one document revision and filter. A drum edit holds tens of thousands
/// of clips: the rows are listed (and filtered) when either changes, not every frame.
#[derive(Clone)]
struct ClipRows {
    revision: u64,
    filter: String,
    kind: String,
    rows: std::sync::Arc<Vec<ClipRow>>,
}

fn clip_rows(s: &soundcraft_model::Session, needle: &str, kind: &str) -> Vec<ClipRow> {
    let shown = |name: &str| needle.is_empty() || name.to_lowercase().contains(needle);
    let source_shown = |src: &soundcraft_model::Source| match kind {
        "all" | "audio" => true,
        "unused" => s.clips_using(src.id).next().is_none(),
        _ => false,
    };
    let clip_shown = |c: &soundcraft_model::Clip| match kind {
        "all" => true,
        "audio" => c.is_audio(),
        "midi" => matches!(c.content, soundcraft_model::ClipContent::Midi { .. }),
        "video" => matches!(c.content, soundcraft_model::ClipContent::Video { .. }),
        "groups" => c.group.is_some(),
        "auto" => c.auto_name.is_some(),
        "unused" => false,
        _ => true,
    };
    // Whole files first (bold), then clips on tracks.
    let sources = s
        .sources
        .iter()
        .enumerate()
        .filter(|(_, src)| source_shown(src) && shown(&format!("{} ({}ch)", src.name, src.channels)))
        .map(|(i, _)| ClipRow::Source(i));
    let clips = s.tracks.iter().enumerate().flat_map(|(track, tr)| {
        tr.clips().iter().enumerate().filter(|(_, c)| clip_shown(c) && shown(&c.name)).map(move |(clip, _)| ClipRow::Clip { track, clip })
    });
    sources.chain(clips).collect()
}

/// Select every source/clip currently visible under the Clip List's two filters.
pub(crate) fn select_all_visible(app: &mut SoundApp) {
    let needle = app.ui.clip_filter.to_lowercase();
    let kind = app.ui.clip_filter_kind.clone();
    let rows = clip_rows(app.engine.session(), &needle, &kind);
    let s = app.engine.session_mut();
    s.edit.selected_clips.clear();
    s.edit.selected_sources.clear();
    for row in rows {
        match row {
            ClipRow::Source(i) => {
                if let Some(id) = s.sources.get(i).map(|src| src.id) {
                    s.edit.selected_sources.push(id);
                }
            }
            ClipRow::Clip { track, clip } => {
                if let Some(id) = s.tracks.get(track).and_then(|t| t.clips().get(clip)).map(|c| c.id) {
                    s.edit.selected_clips.push(id);
                }
            }
        }
    }
}

/// Clips panel with a filter and a revision-cached list of audio sources and clips.
pub fn clip_list(app: &mut SoundApp, ui: &mut Ui) {
    let t = Tokens::current();
    panel_header(ui, "CLIPS");
    let fkey = ui.make_persistent_id("soundcraft.clip_filter");
    ui.add(
        egui::TextEdit::singleline(&mut app.ui.clip_filter)
            .id(fkey)
            .hint_text(crate::localization::text("🔍 Find clips"))
            .desired_width(f32::INFINITY),
    );
    if app.ui.focus_clip_filter {
        ui.ctx().memory_mut(|m| m.request_focus(fkey));
        app.ui.focus_clip_filter = false;
    }
    let offline_count = {
        let s = app.engine.session();
        s.sources.iter().filter(|src| !s.pool.contains(src.id)).count()
    };
    if offline_count > 0
        && ui.button(format!("Load Offline ({offline_count})")).on_hover_text("Load offline audio clips that fit in free RAM").clicked()
    {
        let _ = app.run("session.load_offline", json!({}));
    }
    let filter = app.ui.clip_filter.clone();
    let kind = app.ui.clip_filter_kind.clone();
    let needle = filter.to_lowercase();
    let rkey = egui::Id::unique("clip_rows");
    let revision = app.engine.revision;
    let cached = ui.ctx().memory(|m| m.data.get_temp::<ClipRows>(rkey)).filter(|c| c.revision == revision && c.filter == needle && c.kind == kind);
    let rows = match cached {
        Some(c) => c.rows,
        None => {
            let rows = std::sync::Arc::new(clip_rows(app.engine.session(), &needle, &kind));
            ui.ctx().memory_mut(|m| m.data.insert_temp(rkey, ClipRows { revision, filter: needle, kind, rows: rows.clone() }));
            rows
        }
    };
    let row_h = 17.0;
    let clips_area = with_pending_offset(egui::ScrollArea::vertical().id_salt("clips_scroll"), ui, "clips_scroll");
    clips_area.auto_shrink([false, false]).show_rows(ui, row_h, rows.len(), |ui, shown| {
        let shown_start = shown.start;
        // Only the rows in view: what they show is read now, before any of them acts on the session.
        let items: Vec<(usize, u64, String, bool, [u8; 3], bool)> = {
            let s = app.engine.session();
            rows.get(shown)
                .unwrap_or_default()
                .iter()
                .enumerate()
                .filter_map(|(offset, row)| match *row {
                    ClipRow::Source(i) => s.sources.get(i).map(|src| {
                        let offline = if s.pool.contains(src.id) { "" } else { " [offline]" };
                        (
                            shown_start + offset,
                            src.id.0,
                            format!("{} ({}ch){offline}", src.name, src.channels),
                            true,
                            [150, 150, 150],
                            s.edit.selected_sources.contains(&src.id),
                        )
                    }),
                    ClipRow::Clip { track, clip } => {
                        let tr = s.tracks.get(track)?;
                        let c = tr.clips().get(clip)?;
                        Some((
                            shown_start + offset,
                            c.id.0,
                            c.name.clone(),
                            false,
                            c.color.unwrap_or(tr.color),
                            s.edit.selected_clips.contains(&c.id),
                        ))
                    }
                })
                .collect()
        };
        for (row_index, id, name, whole, color, selected) in items {
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), if whole { Sense::click_and_drag() } else { Sense::click() });
            resp.context_menu(|ui| {
                if whole {
                    if !app.engine.session().pool.contains(SourceId(id)) && ui.button("Load Offline Clip").clicked() {
                        let _ = app.run("session.load_offline", json!({"source": id}));
                        ui.close();
                    }
                    if ui.button(crate::localization::text("Place on New Track")).clicked() {
                        let _ = app.run("clip.place_source", json!({"source": id}));
                        ui.close();
                    }
                    let sel_track = app.engine.session().edit.selected_tracks.first().map(|t| t.0);
                    if let Some(t) = sel_track
                        && ui.button(crate::localization::text("Place on Selected Track")).clicked()
                    {
                        let _ = app.run("clip.place_source", json!({"source": id, "track": t}));
                        ui.close();
                    }
                    ui.separator();
                }
                if !whole && ui.button(crate::localization::text("Auto Rename…")).clicked() {
                    // The selected clips when the row is one of them, otherwise the row's clip.
                    let clips = if selected { app.engine.session().edit.selected_clips.clone() } else { vec![ClipId(id)] };
                    app.dialogs.open_auto_rename(&app.engine, clips);
                    ui.close();
                }
                if ui.button(crate::localization::text("Clear…")).clicked() {
                    let (clips, sources) = if whole { (vec![], vec![SourceId(id)]) } else { (vec![ClipId(id)], vec![]) };
                    app.dialogs.open_clear(&app.engine, clips, sources);
                    ui.close();
                }
            });
            if whole {
                // Whole files drag onto tracks.
                resp.dnd_set_drag_payload(crate::DragSource(id));
                if resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            }
            if selected {
                ui.painter().rect_filled(r, 0.0, t.row_selected);
            }
            ui.painter().rect_filled(Rect::from_min_size(pos2(r.min.x + 6.0, r.min.y + 4.0), vec2(9.0, 9.0)), 1.0, rgb(color));
            ui.painter().with_clip_rect(r).text(
                pos2(r.min.x + 22.0, r.center().y),
                Align2::LEFT_CENTER,
                &name,
                if whole { bold(11.0) } else { regular(11.0) },
                t.text,
            );
            let alt_down = ui.input(|i| i.modifiers.alt);
            if alt_down && resp.is_pointer_button_down_on() {
                if whole {
                    app.start_source_audition(SourceId(id));
                } else {
                    app.start_clip_audition(ClipId(id));
                }
            }
            if resp.clicked() {
                let mods = ui.input(|i| i.modifiers);
                let _ = app.run("options.keyboard_focus", json!({"focus": "clips"}));
                if resp.double_clicked() && !whole && !mods.alt && !mods.ctrl && !mods.command && !mods.shift {
                    if let Some((_, clip)) = app.engine.session().find_clip(ClipId(id)) {
                        app.dialogs.open_rename_clip(clip.id, &clip.name);
                    }
                    continue;
                }
                // Pro Tools' Option-Control/Alt-Start click spots the Clip List item at
                // the current edit insertion. Reuse SoundCraft's placement command; for
                // a timeline clip, keep its current track as the target when possible.
                if mods.alt && (mods.ctrl || mods.command) && !mods.shift {
                    spot_clip_list_row(app, id, whole);
                } else if !mods.alt {
                    select_clip_list_row(app, rows.as_ref(), row_index, id, whole, mods);
                }
            }
        }
    });
    if app.clip_audition.is_some() && !ui.input(|i| i.pointer.primary_down()) {
        app.stop_clip_audition();
    }
}

fn spot_clip_list_row(app: &mut SoundApp, id: u64, whole: bool) {
    let target = {
        let s = app.engine.session();
        if whole {
            Some((SourceId(id), None))
        } else {
            s.find_clip(ClipId(id)).and_then(|(track, clip)| clip.source().map(|source| (source, Some(track.0))))
        }
    };
    let Some((source, track)) = target else { return };
    let mut params = json!({"source": source.0});
    if let Some(track) = track {
        params["track"] = json!(track);
    }
    let _ = app.run("clip.place_source", params);
}

/// Apply the Clip List's ordinary multi-selection modifiers without changing the timeline range.
fn select_clip_list_row(app: &mut SoundApp, rows: &[ClipRow], row_index: usize, id: u64, whole: bool, mods: egui::Modifiers) {
    if !mods.command && !mods.shift {
        let _ = app.run("edit.select", if whole { json!({"sources": [id]}) } else { json!({"clips": [id]}) });
        return;
    }
    let s = app.engine.session();
    let anchor = rows.iter().enumerate().find_map(|(index, row)| match *row {
        ClipRow::Source(i) if s.sources.get(i).is_some_and(|source| s.edit.selected_sources.contains(&source.id)) => Some(index),
        ClipRow::Clip { track, clip }
            if s.tracks.get(track).and_then(|track| track.clips().get(clip)).is_some_and(|clip| s.edit.selected_clips.contains(&clip.id)) =>
        {
            Some(index)
        }
        _ => None,
    });
    let range = if mods.shift {
        anchor.map_or_else(|| row_index..=row_index, |start| start.min(row_index)..=start.max(row_index))
    } else {
        row_index..=row_index
    };
    let mut sources = if mods.command { s.edit.selected_sources.clone() } else { Vec::new() };
    let mut clips = if mods.command { s.edit.selected_clips.clone() } else { Vec::new() };
    for row in rows.get(range).into_iter().flatten() {
        match *row {
            ClipRow::Source(i) => {
                if let Some(source) = s.sources.get(i).map(|source| source.id)
                    && !sources.contains(&source)
                {
                    sources.push(source);
                }
            }
            ClipRow::Clip { track, clip } => {
                if let Some(clip) = s.tracks.get(track).and_then(|track| track.clips().get(clip)).map(|clip| clip.id)
                    && !clips.contains(&clip)
                {
                    clips.push(clip);
                }
            }
        }
    }
    if mods.command && !mods.shift {
        if whole {
            sources.retain(|source| *source != SourceId(id));
        } else {
            clips.retain(|clip| *clip != ClipId(id));
        }
    }
    let edit = &mut app.engine.session_mut().edit;
    edit.selected_sources = sources;
    edit.selected_clips = clips;
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundcraft_model::{ChannelFormat, Clip, Session, TrackKind};

    #[test]
    fn clip_rows_list_every_clip_in_track_order_and_filter_by_name() {
        let mut s = Session::default();
        for names in [["Kick 1", "Snare 1"], ["Kick 2", "Tom"]] {
            let t = s.add_track(TrackKind::Audio, ChannelFormat::Mono, None);
            for (i, n) in names.into_iter().enumerate() {
                let c = Clip::audio(s.new_clip_id(), n, SourceId(1), 0, i as i64 * 100, 100);
                s.track_mut(t).unwrap().playlist_mut().unwrap().clips.push(c);
            }
        }
        let names = |rows: Vec<ClipRow>| -> Vec<String> {
            rows.into_iter()
                .filter_map(|r| match r {
                    ClipRow::Clip { track, clip } => Some(s.tracks[track].clips()[clip].name.clone()),
                    ClipRow::Source(_) => None,
                })
                .collect()
        };
        assert_eq!(names(clip_rows(&s, "", "all")), ["Kick 1", "Snare 1", "Kick 2", "Tom"]);
        assert_eq!(names(clip_rows(&s, "kick", "all")), ["Kick 1", "Kick 2"]);
        assert!(clip_rows(&s, "cowbell", "all").is_empty());
    }
}
