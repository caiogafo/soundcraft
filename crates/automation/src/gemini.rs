//! Gemini Interactions API client for controlling SoundCraft and analyzing audio.
//!
//! Defaults to `gemini-4-argon` and uses the `GEMINI_API_KEY` (provided at runtime, never stored
//! in the repository).

use crate::{Backend, mcp};
use base64::Engine as _;
use serde_json::{Value, json};
use std::path::Path;

pub const DEFAULT_MODEL: &str = "gemini-4-argon";
pub const INTERACTIONS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";
const MAX_TOOL_ROUNDS: usize = 12;
const MAX_INLINE_AUDIO_BYTES: u64 = 20 * 1024 * 1024;

const SYSTEM_INSTRUCTION: &str = "You are SoundCraft's AI audio assistant. You can inspect the open DAW session, \
execute SoundCraft commands through the provided tools, and listen to attached audio files. \
Always inspect the session before making assumptions about track names or clip positions, \
and summarize the exact actions you took or the audio observations clearly.";

/// Configuration for a Gemini agent request.
#[derive(Clone, Debug)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub system_instruction: String,
}

impl GeminiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), model: DEFAULT_MODEL.to_string(), system_instruction: SYSTEM_INSTRUCTION.to_string() }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let model = model.into();
        if !model.trim().is_empty() {
            self.model = model;
        }
        self
    }
}

/// Convert SoundCraft's MCP tool definitions into Gemini Interactions `function` tool declarations.
pub fn gemini_tools() -> Vec<Value> {
    mcp::tool_definitions()
        .into_iter()
        .map(|t| {
            json!({
                "type": "function",
                "name": t["name"],
                "description": t["description"],
                "parameters": t["inputSchema"],
            })
        })
        .collect()
}

/// Map an audio file extension to a MIME type accepted by Gemini multimodal audio input.
pub fn audio_mime_type(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "wav" => Some("audio/wav"),
        "mp3" => Some("audio/mp3"),
        "aiff" | "aif" => Some("audio/aiff"),
        "aac" | "m4a" => Some("audio/aac"),
        "ogg" | "oga" => Some("audio/ogg"),
        "flac" => Some("audio/flac"),
        _ => None,
    }
}

fn build_input(prompt: &str, audio: Option<(&[u8], &str)>) -> Result<Value, String> {
    let Some((bytes, mime)) = audio else {
        return Ok(Value::String(prompt.to_string()));
    };
    if bytes.len() as u64 > MAX_INLINE_AUDIO_BYTES {
        return Err(format!("audio exceeds {} MiB inline limit", MAX_INLINE_AUDIO_BYTES / (1024 * 1024)));
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(json!([
        { "type": "text", "text": prompt },
        { "type": "audio", "data": b64, "mime_type": mime }
    ]))
}

/// Read `name` from a `.env` file in the current working directory when present.
pub fn read_dotenv_key(name: &str) -> Option<String> {
    let text = std::fs::read_to_string(".env").ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim();
        if let Some((k, v)) = line.split_once('=')
            && k.trim() == name
        {
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

fn post_interaction(api_key: &str, body: &Value) -> Result<Value, String> {
    let payload = serde_json::to_vec(body).map_err(|e| format!("failed to serialize Gemini request: {e}"))?;
    let agent: ureq::Agent = ureq::Agent::config_builder().http_status_as_error(false).build().into();
    let mut response = agent
        .post(INTERACTIONS_URL)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .send(payload)
        .map_err(|e| format!("Gemini API request failed: {e}"))?;
    let status = response.status();
    let bytes = response.body_mut().read_to_vec().map_err(|e| format!("Gemini API response read failed: {e}"))?;
    if !status.is_success() {
        let text = String::from_utf8_lossy(&bytes);
        return Err(format!("Gemini API HTTP {}: {}", status.as_u16(), text.trim()));
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("Gemini API response parse failed: {e}"))
}

fn extract_function_calls(interaction: &Value) -> Vec<(String, String, Value)> {
    let Some(steps) = interaction.get("steps").and_then(Value::as_array) else {
        return Vec::new();
    };
    steps
        .iter()
        .filter(|s| s.get("type").and_then(Value::as_str) == Some("function_call"))
        .filter_map(|s| {
            let id = s.get("id").and_then(Value::as_str)?.to_string();
            let name = s.get("name").and_then(Value::as_str)?.to_string();
            let args = s.get("arguments").cloned().unwrap_or_else(|| json!({}));
            Some((id, name, args))
        })
        .collect()
}

fn extract_output_text(interaction: &Value) -> String {
    if let Some(text) = interaction.get("output_text").and_then(Value::as_str)
        && !text.is_empty()
    {
        return text.to_string();
    }
    let Some(steps) = interaction.get("steps").and_then(Value::as_array) else {
        return String::new();
    };
    let mut out = Vec::new();
    for step in steps {
        if step.get("type").and_then(Value::as_str) != Some("model_output") {
            continue;
        }
        if let Some(parts) = step.get("content").and_then(Value::as_array) {
            for part in parts {
                if part.get("type").and_then(Value::as_str) == Some("text")
                    && let Some(t) = part.get("text").and_then(Value::as_str)
                {
                    out.push(t);
                }
            }
        }
    }
    out.join("")
}

/// Ask Gemini (`gemini-4-argon` by default) to inspect/edit SoundCraft or analyze `audio_path`.
pub fn ask<B: Backend>(
    server: &mut mcp::Server<B>,
    cfg: &GeminiConfig,
    prompt: &str,
    audio_path: Option<&Path>,
    on_tool: impl FnMut(&str, &Value),
) -> Result<String, String> {
    let owned = match audio_path {
        Some(path) => {
            let mime = audio_mime_type(path).ok_or_else(|| format!("{}: unsupported audio extension for Gemini", path.display()))?;
            let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
            if !meta.is_file() {
                return Err(format!("{}: not a regular file", path.display()));
            }
            if meta.len() > MAX_INLINE_AUDIO_BYTES {
                return Err(format!("{}: exceeds {} MiB inline audio limit", path.display(), MAX_INLINE_AUDIO_BYTES / (1024 * 1024)));
            }
            let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Some((bytes, mime))
        }
        None => None,
    };
    let audio_ref = owned.as_ref().map(|(b, m)| (b.as_slice(), *m));
    ask_with_audio_bytes(server, cfg, prompt, audio_ref, on_tool)
}

/// Ask Gemini (`gemini-4-argon` by default) with optional in-memory audio bytes `(bytes, mime_type)`.
pub fn ask_with_audio_bytes<B: Backend>(
    server: &mut mcp::Server<B>,
    cfg: &GeminiConfig,
    prompt: &str,
    audio: Option<(&[u8], &str)>,
    mut on_tool: impl FnMut(&str, &Value),
) -> Result<String, String> {
    if cfg.api_key.trim().is_empty() {
        return Err("missing Gemini API key (set GEMINI_API_KEY, .env, or pass --api-key)".into());
    }
    let tools = gemini_tools();
    let mut input = build_input(prompt, audio)?;
    let mut previous_id: Option<String> = None;

    for _ in 0..MAX_TOOL_ROUNDS {
        let mut body = json!({
            "model": cfg.model,
            "input": input,
            "tools": tools,
            "system_instruction": cfg.system_instruction,
        });
        if let Some(prev) = &previous_id
            && let Some(obj) = body.as_object_mut()
        {
            obj.insert("previous_interaction_id".into(), Value::String(prev.clone()));
        }

        let interaction = post_interaction(&cfg.api_key, &body)?;
        if let Some(err) = interaction.get("error").filter(|e| !e.is_null()) {
            return Err(format!("Gemini error: {err}"));
        }
        let calls = extract_function_calls(&interaction);
        if calls.is_empty() {
            return Ok(extract_output_text(&interaction));
        }
        previous_id = interaction.get("id").and_then(Value::as_str).map(str::to_string);
        let mut results = Vec::with_capacity(calls.len());
        for (call_id, name, args) in calls {
            on_tool(&name, &args);
            let result = server.call_tool(&name, &args);
            results.push(json!({
                "type": "function_result",
                "name": name,
                "call_id": call_id,
                "result": [{ "type": "text", "text": result.to_string() }],
            }));
        }
        input = Value::Array(results);
    }
    Err(format!("exceeded {MAX_TOOL_ROUNDS} tool-call rounds without a final answer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_gemini_4_argon() {
        assert_eq!(GeminiConfig::new("test-key").model, "gemini-4-argon");
    }

    #[test]
    fn tools_wrap_mcp_definitions_as_functions() {
        let tools = gemini_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().all(|t| t["type"] == "function" && t["name"].is_string() && t["parameters"].is_object()));
        assert!(tools.iter().any(|t| t["name"] == "execute"));
        assert!(tools.iter().any(|t| t["name"] == "inspect_session"));
    }

    #[test]
    fn audio_mime_types_cover_common_formats() {
        assert_eq!(audio_mime_type(Path::new("voice.wav")), Some("audio/wav"));
        assert_eq!(audio_mime_type(Path::new("mix.MP3")), Some("audio/mp3"));
        assert_eq!(audio_mime_type(Path::new("stem.flac")), Some("audio/flac"));
        assert_eq!(audio_mime_type(Path::new("notes.txt")), None);
    }

    #[test]
    fn step_parsers_extract_calls_and_output_text() {
        let interaction = json!({
            "id": "int_123",
            "steps": [
                { "type": "function_call", "id": "fc_1", "name": "inspect_session", "arguments": {"detail": "summary"} },
                { "type": "model_output", "content": [{ "type": "text", "text": "Done!" }] }
            ]
        });
        let calls = extract_function_calls(&interaction);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "fc_1");
        assert_eq!(calls[0].1, "inspect_session");
        assert_eq!(extract_output_text(&interaction), "Done!");
    }
}
