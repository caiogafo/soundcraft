use super::args::AskArgs;
use super::error::CliError;
use soundcraft_automation::{Remote, gemini::GeminiConfig, mcp::Server};
use soundcraft_engine::Engine;
use std::io::Write;

pub(crate) fn ask(args: &AskArgs, out: &mut dyn Write, err: &mut dyn Write) -> Result<(), CliError> {
    let dotenv = if args.api_key.is_none() { soundcraft_automation::gemini::read_dotenv_key("GEMINI_API_KEY") } else { None };
    let api_key = args.api_key.as_deref().or(dotenv.as_deref()).unwrap_or("");
    let cfg = GeminiConfig::new(api_key).with_model(&args.model);
    let audio = args.audio.as_deref();
    let reply = if let Some(target) = &args.connect {
        let mut server = Server::new(Remote::new(target));
        soundcraft_automation::gemini::ask(&mut server, &cfg, &args.prompt, audio, |name, tool_args| {
            outln!(err, "gemini tool -> {name} {tool_args}");
        })
    } else {
        let e = if args.demo { soundcraft_engine::demo::demo_engine() } else { Engine::default() };
        let mut server = Server::new(soundcraft_automation::Headless::new(e));
        soundcraft_automation::gemini::ask(&mut server, &cfg, &args.prompt, audio, |name, tool_args| {
            outln!(err, "gemini tool -> {name} {tool_args}");
        })
    }
    .map_err(|e| CliError::Serve(std::io::Error::other(e)))?;
    outln!(out, "{reply}");
    Ok(())
}
