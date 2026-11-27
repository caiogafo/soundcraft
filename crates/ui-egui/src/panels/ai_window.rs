use crate::SoundApp;
use crate::theme::{Tokens, bold, mono, regular};
use egui::{Color32, RichText, ScrollArea, vec2};
use serde_json::Value;
use soundcraft_model::Session;
use soundcraft_time::Range;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

const MAX_SELECTION_SECS: f64 = 600.0;

#[derive(Clone)]
struct ChatEntry {
    is_user: bool,
    text: String,
    audio_note: Option<String>,
}

struct AiReply {
    result: Result<String, String>,
    tool_calls: Vec<(String, Value)>,
}

#[derive(Default)]
struct AiRuntime {
    prompt: String,
    api_key_override: String,
    send_selection_audio: bool,
    history: Vec<ChatEntry>,
    pending: Option<Receiver<AiReply>>,
}

fn state_slot(ctx: &egui::Context) -> Arc<Mutex<AiRuntime>> {
    let id = egui::Id::unique("ai_assistant_runtime");
    ctx.data_mut(|d| {
        d.get_temp::<Arc<Mutex<AiRuntime>>>(id).unwrap_or_else(|| {
            let slot = Arc::new(Mutex::new(AiRuntime { send_selection_audio: true, ..Default::default() }));
            d.insert_temp(id, slot.clone());
            slot
        })
    })
}

fn resolve_api_key(override_key: &str) -> String {
    if !override_key.trim().is_empty() {
        return override_key.trim().to_string();
    }
    if let Ok(v) = std::env::var("GEMINI_API_KEY")
        && !v.trim().is_empty()
    {
        return v.trim().to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(v) = soundcraft_automation::gemini::read_dotenv_key("GEMINI_API_KEY") {
        return v;
    }
    String::new()
}

fn render_selection_wav(session: &Session, range: Range) -> Result<(Vec<u8>, f64), String> {
    let sr = session.sample_rate.hz().max(1);
    let len = range.end.saturating_sub(range.start);
    if len <= 0 {
        return Err("select a range on the timeline first".into());
    }
    let secs = len as f64 / f64::from(sr);
    if secs > MAX_SELECTION_SECS {
        return Err(format!("selection is {secs:.1} s (max {MAX_SELECTION_SECS:.0} s)"));
    }
    let rendered = soundcraft_mix::render_range(session, range, 512);
    let frames = rendered.first().map_or(0, Vec::len);
    if frames == 0 {
        return Err("selection rendered 0 frames".into());
    }
    let n_ch = rendered.len().max(1) as f32;
    let mut mono = vec![0.0_f32; frames];
    for ch in &rendered {
        for (dst, &s) in mono.iter_mut().zip(ch.iter()) {
            *dst += s / n_ch;
        }
    }
    let resampled = if sr == 16_000 { vec![mono] } else { soundcraft_dsp::offline::resample(&[mono], sr, 16_000) };
    let buf = craft_audio::AudioBuffer { sample_rate: 16_000, channels: resampled };
    let opts = craft_audio::EncodeOptions::new(craft_audio::EncodeFormat::Wav).with_bit_depth(craft_audio::BitDepth::Int16);
    let wav = craft_audio::encode(&buf, &opts).map_err(|e| e.to_string())?;
    Ok((wav, secs))
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_request(
    ctx: egui::Context,
    session_snapshot: Arc<Session>,
    api_key: String,
    prompt: String,
    wav_audio: Option<Vec<u8>>,
) -> Receiver<AiReply> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let engine = soundcraft_engine::Engine::new((*session_snapshot).clone());
        let mut server = soundcraft_automation::mcp::Server::new(soundcraft_automation::Headless::new(engine));
        let cfg = soundcraft_automation::gemini::GeminiConfig::new(api_key);
        let audio_ref = wav_audio.as_deref().map(|b| (b, "audio/wav"));
        let mut tool_calls = Vec::new();
        let result = soundcraft_automation::gemini::ask_with_audio_bytes(&mut server, &cfg, &prompt, audio_ref, |name, args| {
            if name == "execute"
                && let Some(cmd) = args.get("command").and_then(Value::as_str)
            {
                let params = args.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));
                tool_calls.push((cmd.to_string(), params));
            } else if name == "batch"
                && let Some(calls) = args.get("calls").and_then(Value::as_array)
            {
                for c in calls {
                    if let Some(cmd) = c.get("command").and_then(Value::as_str) {
                        let params = c.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));
                        tool_calls.push((cmd.to_string(), params));
                    }
                }
            }
        });
        let _ = tx.send(AiReply { result, tool_calls });
        ctx.request_repaint();
    });
    rx
}

#[cfg(target_arch = "wasm32")]
fn spawn_request(
    ctx: egui::Context,
    _session_snapshot: Arc<Session>,
    _api_key: String,
    _prompt: String,
    _wav_audio: Option<Vec<u8>>,
) -> Receiver<AiReply> {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = tx.send(AiReply { result: Err("AI Assistant is not supported in the browser build".into()), tool_calls: Vec::new() });
    ctx.request_repaint();
    rx
}

pub(super) fn ai_window(app: &mut SoundApp, ctx: &egui::Context) {
    if !app.extra.show_ai_assistant {
        return;
    }
    let slot = state_slot(ctx);
    let mut pending_commands: Vec<(String, Value)> = Vec::new();
    let mut open = app.extra.show_ai_assistant;
    let t = Tokens::current();

    // Poll any in-flight background response.
    if let Ok(mut rt) = slot.lock()
        && let Some(rx) = &rt.pending
    {
        match rx.try_recv() {
            Ok(reply) => {
                rt.pending = None;
                match reply.result {
                    Ok(text) => {
                        pending_commands = reply.tool_calls;
                        rt.history.push(ChatEntry { is_user: false, text, audio_note: None });
                    }
                    Err(err) => {
                        rt.history.push(ChatEntry { is_user: false, text: format!("Error: {err}"), audio_note: None });
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                rt.pending = None;
                rt.history.push(ChatEntry { is_user: false, text: "Error: background worker stopped".into(), audio_note: None });
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
    }

    for (cmd, params) in pending_commands {
        let _ = app.run(&cmd, params);
    }

    let sel = app.engine.session().edit.selection;
    let sr = app.engine.session().sample_rate.hz().max(1);
    let sel_secs = (sel.end.saturating_sub(sel.start)).max(0) as f64 / f64::from(sr);
    let has_sel = sel.end > sel.start;

    egui::Window::new("AI Assistant (gemini-4-argon)")
        .id(egui::Id::unique("AI Assistant"))
        .open(&mut open)
        .default_size(vec2(460.0, 480.0))
        .resizable(true)
        .show(ctx, |ui| {
            let Ok(mut rt) = slot.lock() else { return };
            let busy = rt.pending.is_some();

            ui.horizontal(|ui| {
                let has_key = !resolve_api_key(&rt.api_key_override).is_empty();
                let status = if has_key { "API key: ready" } else { "API key: missing" };
                let color = if has_key { Color32::from_rgb(100, 200, 120) } else { Color32::from_rgb(230, 110, 90) };
                ui.label(RichText::new(status).font(mono(11.0)).color(color));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() && !busy {
                        rt.history.clear();
                    }
                });
            });

            if resolve_api_key(&rt.api_key_override).is_empty() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("GEMINI_API_KEY:").font(regular(11.5)));
                    ui.add(egui::TextEdit::singleline(&mut rt.api_key_override).password(true).hint_text("or set in .env"));
                });
            }

            ui.separator();

            ScrollArea::vertical().auto_shrink([false, false]).max_height(300.0).stick_to_bottom(true).show(ui, |ui| {
                if rt.history.is_empty() {
                    ui.label(
                        RichText::new("Select an excerpt on the timeline (optional) and ask gemini-4-argon to analyze it or edit the session.")
                            .font(regular(12.0))
                            .color(t.mute),
                    );
                }
                for msg in &rt.history {
                    let who = if msg.is_user { "You" } else { "gemini-4-argon" };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(who).font(bold(11.5)).color(if msg.is_user { t.accent } else { t.text }));
                        if let Some(note) = &msg.audio_note {
                            ui.label(RichText::new(note).font(mono(10.5)).color(t.mute));
                        }
                    });
                    ui.add(egui::Label::new(RichText::new(&msg.text).font(regular(12.0))).selectable(true));
                    ui.add_space(6.0);
                }
                if busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Analyzing with gemini-4-argon…").font(regular(11.5)).color(t.mute));
                    });
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(&mut rt.send_selection_audio, "Send selection audio");
                let sel_label = if has_sel { format!("({sel_secs:.1} s)") } else { "(no timeline selection)".into() };
                ui.label(RichText::new(sel_label).font(mono(11.0)).color(t.mute));
            });

            let mut send_now = false;
            ui.horizontal(|ui| {
                let resp = ui.add_sized(
                    vec2(ui.available_width() - 68.0, 52.0),
                    egui::TextEdit::multiline(&mut rt.prompt).hint_text("Ask about the selected audio or tell SoundCraft what to do…"),
                );
                if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                    send_now = true;
                }
                if ui.add_enabled(!busy && !rt.prompt.trim().is_empty(), egui::Button::new("Send").min_size(vec2(60.0, 52.0))).clicked() {
                    send_now = true;
                }
            });

            if send_now && !busy && !rt.prompt.trim().is_empty() {
                let prompt = rt.prompt.trim().to_string();
                rt.prompt.clear();
                let key = resolve_api_key(&rt.api_key_override);
                let (wav_opt, audio_note) = if rt.send_selection_audio && has_sel {
                    match render_selection_wav(app.engine.session(), sel) {
                        Ok((wav, secs)) => (Some(wav), Some(format!("[audio: {secs:.1} s]"))),
                        Err(e) => {
                            rt.history.push(ChatEntry { is_user: true, text: prompt, audio_note: None });
                            rt.history.push(ChatEntry { is_user: false, text: format!("Audio render error: {e}"), audio_note: None });
                            return;
                        }
                    }
                } else {
                    (None, None)
                };
                rt.history.push(ChatEntry { is_user: true, text: prompt.clone(), audio_note });
                rt.pending = Some(spawn_request(ctx.clone(), app.engine.session_arc(), key, prompt, wav_opt));
            }
        });

    app.extra.show_ai_assistant = open;
}
