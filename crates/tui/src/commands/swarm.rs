//! `/swarm` and `/swarm-big` slash commands: orchestrator-led multi-agent modes.
//!
//! Activating swarm mode flips a session-level flag (`App::swarm_active`)
//! and records the selected scale (`App::swarm_mode`).
//! While active, every outbound user message is wrapped with the standing
//! orchestrator brief (`prompts::swarm_orchestrator_wrap`) so the
//! assistant takes on the orchestrator role: decompose the request,
//! dispatch parallel `agent_open` workers, gather results via
//! `agent_eval`, verify side effects, and integrate a single answer.
//!
//! The wrapper text is byte-stable across turns, so DeepSeek's automatic
//! prefix cache keeps hitting on the system prompt + tool list + prior
//! turns. The orchestrator brief itself explicitly instructs workers to
//! use cache-aware patterns (`fork_context: false` by default, stable
//! worker session names, `resident_file` for repeated single-file work,
//! `handle_read` instead of re-quoting transcripts).
//!
//! Sub-commands:
//!
//! - `/swarm` / `/swarm-big`              — toggle on/off
//! - `/swarm on` / `/swarm-big on`        — activate
//! - `/swarm off` / `/swarm-big off`      — deactivate
//! - `/swarm status`                      — report current state and session brief
//! - `/swarm brief <text>`                — pin a session brief surfaced inside the wrapper
//! - `/swarm brief clear`                 — clear the pinned brief
//! - `/swarm <task>` / `/swarm-big <task>` — activate (if needed) and immediately send
//!   `<task>` through the orchestrator
//!
//! Anything that doesn't match a sub-command is treated as a one-shot
//! task — the swarm activates and the orchestrator gets the user's text
//! on the next turn.

use crate::tui::app::{App, AppAction, SwarmMode};

use super::CommandResult;

const SWARM_USAGE: &str = "Usage: /swarm [on|off|status|help|brief <text>|brief clear|<task>]";
const SWARM_BIG_USAGE: &str =
    "Usage: /swarm-big [on|off|status|help|brief <text>|brief clear|<task>]";

pub fn swarm(app: &mut App, args: Option<&str>) -> CommandResult {
    handle_swarm(app, args, SwarmMode::Standard)
}

pub fn swarm_big(app: &mut App, args: Option<&str>) -> CommandResult {
    handle_swarm(app, args, SwarmMode::Big)
}

fn handle_swarm(app: &mut App, args: Option<&str>, mode: SwarmMode) -> CommandResult {
    let raw = args.unwrap_or("").trim();

    if raw.is_empty() {
        return toggle(app, mode);
    }

    let mut parts = raw.splitn(2, char::is_whitespace);
    let head = parts.next().unwrap_or("").to_ascii_lowercase();
    let tail = parts.next().map(str::trim).filter(|s| !s.is_empty());

    match head.as_str() {
        "on" | "enable" | "start" if tail.is_none() => activate(app, mode, None),
        "off" | "disable" | "stop" | "end" if tail.is_none() => deactivate(app, mode),
        "status" | "show" | "?" if tail.is_none() => CommandResult::message(status_text(app)),
        "help" if tail.is_none() => {
            CommandResult::message(format!("{}\n\n{}", usage(mode), help_text(mode)))
        }
        "brief" => handle_brief(app, tail),
        _ => activate(app, mode, Some(raw.to_string())),
    }
}

fn toggle(app: &mut App, mode: SwarmMode) -> CommandResult {
    if app.swarm_active && app.swarm_mode == mode {
        deactivate(app, mode)
    } else {
        activate(app, mode, None)
    }
}

fn activate(app: &mut App, mode: SwarmMode, task: Option<String>) -> CommandResult {
    let was_active = app.swarm_active;
    let was_same_mode = was_active && app.swarm_mode == mode;
    app.swarm_active = true;
    app.swarm_mode = mode;
    let brief_note = match app.swarm_brief.as_deref() {
        Some(brief) if !brief.trim().is_empty() => format!(" (brief: {})", short_brief(brief)),
        _ => String::new(),
    };

    if let Some(task) = task {
        // One-shot: activate and dispatch the task in the same turn.
        // The dispatch path will inject the orchestrator wrapper because
        // swarm_active is now true. We do NOT pre-wrap here so the
        // assistant's transcript "User" cell shows the raw request.
        let header = if was_same_mode {
            format!(
                "{} active{brief_note}; dispatching orchestrator.",
                mode.label()
            )
        } else {
            format!(
                "{} activated{brief_note}; dispatching orchestrator.",
                mode.label()
            )
        };
        return CommandResult::with_message_and_action(header, AppAction::SendMessage(task));
    }

    if was_same_mode {
        CommandResult::message(format!("{} already active{brief_note}.", mode.label()))
    } else {
        CommandResult::message(format!(
            "{} activated{brief_note}. Next prompt will be handled by the orchestrator. Run `{} off` to leave.",
            mode.label(),
            mode.command()
        ))
    }
}

fn deactivate(app: &mut App, mode: SwarmMode) -> CommandResult {
    if !app.swarm_active {
        return CommandResult::message(format!("{} is already off.", mode.label()));
    }
    let deactivated_mode = app.swarm_mode;
    app.swarm_active = false;
    CommandResult::message(format!(
        "{} deactivated. Subsequent prompts go straight to the model. Use `{}` to re-enable.",
        deactivated_mode.label(),
        deactivated_mode.command()
    ))
}

fn handle_brief(app: &mut App, tail: Option<&str>) -> CommandResult {
    match tail {
        None => {
            // No argument — show current brief or usage hint.
            match app.swarm_brief.as_deref() {
                Some(brief) if !brief.trim().is_empty() => CommandResult::message(format!(
                    "Swarm brief: {brief}\n\nClear with `/swarm brief clear`."
                )),
                _ => CommandResult::message(
                    "No swarm brief pinned. Set one with `/swarm brief <text>`.",
                ),
            }
        }
        Some(value)
            if value.eq_ignore_ascii_case("clear") || value.eq_ignore_ascii_case("none") =>
        {
            if app.swarm_brief.take().is_some() {
                CommandResult::message("Swarm brief cleared.")
            } else {
                CommandResult::message("No swarm brief was set.")
            }
        }
        Some(value) => {
            app.swarm_brief = Some(value.to_string());
            CommandResult::message(format!("Swarm brief pinned: {}", short_brief(value)))
        }
    }
}

fn status_text(app: &App) -> String {
    let mut out = String::from(if app.swarm_active {
        format!("{} mode: ON\n", app.swarm_mode.label())
    } else {
        "Swarm mode: OFF\n".to_string()
    });
    match app.swarm_brief.as_deref() {
        Some(brief) if !brief.trim().is_empty() => {
            out.push_str("Session brief: ");
            out.push_str(brief);
            out.push('\n');
        }
        _ => {
            out.push_str("Session brief: (none)\n");
        }
    }
    out.push_str("\nWhen ON, the selected orchestrator brief is wrapped around every user message so the assistant decomposes the task and dispatches parallel `agent_open` workers. Sub-commands: on | off | status | help | brief <text> | brief clear | <task>.");
    out
}

fn usage(mode: SwarmMode) -> &'static str {
    match mode {
        SwarmMode::Standard => SWARM_USAGE,
        SwarmMode::Big => SWARM_BIG_USAGE,
    }
}

fn help_text(mode: SwarmMode) -> &'static str {
    match mode {
        SwarmMode::Standard => {
            "Swarm orchestrator mode wraps each user turn with a standing brief that tells \
             the assistant to decompose the task, dispatch a few parallel `agent_open` workers \
             (with stable session names + cache-aware `fork_context`/`resident_file` defaults), \
             gather results with `agent_eval`, verify side effects, and integrate one answer. \
             The wrapper text is byte-stable across turns so DeepSeek's prefix cache keeps \
             hitting on the system prompt and prior history."
        }
        SwarmMode::Big => {
            "Big swarm orchestrator mode wraps each user turn with a standing brief that tells \
             the assistant to decompose the task, dispatch 10-100 parallel `agent_open` workers \
             plus an orchestrator when runtime capacity allows, preserve the same stable session \
             names + cache-aware `fork_context`/`resident_file` defaults, gather with \
             `agent_eval`, verify side effects, and integrate one answer. The wrapper text is \
             byte-stable across turns so DeepSeek's prefix cache keeps hitting on the system \
             prompt and prior history."
        }
    }
}

fn short_brief(text: &str) -> String {
    const MAX_CHARS: usize = 80;
    let trimmed = text.trim();
    if trimmed.chars().count() <= MAX_CHARS {
        trimmed.to_string()
    } else {
        let head: String = trimmed.chars().take(MAX_CHARS).collect();
        format!("{head}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tui::app::TuiOptions;
    use std::path::PathBuf;

    fn app() -> App {
        App::new(
            TuiOptions {
                model: "deepseek-v4-pro".to_string(),
                workspace: PathBuf::from("."),
                config_path: None,
                config_profile: None,
                allow_shell: false,
                use_alt_screen: false,
                use_mouse_capture: false,
                use_bracketed_paste: true,
                max_subagents: 2,
                skills_dir: PathBuf::from("."),
                memory_path: PathBuf::from("memory.md"),
                notes_path: PathBuf::from("notes.txt"),
                mcp_config_path: PathBuf::from("mcp.json"),
                use_memory: false,
                start_in_agent_mode: false,
                skip_onboarding: true,
                yolo: false,
                resume_session_id: None,
                initial_input: None,
            },
            &Config::default(),
        )
    }

    #[test]
    fn defaults_to_inactive() {
        let app = app();
        assert!(!app.swarm_active);
        assert_eq!(app.swarm_mode, SwarmMode::Standard);
        assert!(app.swarm_brief.is_none());
    }

    #[test]
    fn toggle_flips_state() {
        let mut app = app();
        let on = swarm(&mut app, None);
        assert!(app.swarm_active, "no-arg /swarm should activate");
        assert_eq!(app.swarm_mode, SwarmMode::Standard);
        assert!(on.action.is_none());
        let off = swarm(&mut app, None);
        assert!(!app.swarm_active, "second /swarm should deactivate");
        assert!(off.action.is_none());
    }

    #[test]
    fn swarm_big_toggles_big_mode() {
        let mut app = app();
        let on = swarm_big(&mut app, None);
        assert!(app.swarm_active, "no-arg /swarm-big should activate");
        assert_eq!(app.swarm_mode, SwarmMode::Big);
        assert!(on.action.is_none());
        let off = swarm_big(&mut app, None);
        assert!(!app.swarm_active, "second /swarm-big should deactivate");
        assert!(off.action.is_none());
    }

    #[test]
    fn on_off_set_explicit_state() {
        let mut app = app();
        swarm(&mut app, Some("on"));
        assert!(app.swarm_active);
        swarm(&mut app, Some("on"));
        assert!(app.swarm_active);
        swarm(&mut app, Some("off"));
        assert!(!app.swarm_active);
        swarm(&mut app, Some("off"));
        assert!(!app.swarm_active);
    }

    #[test]
    fn one_shot_task_activates_and_dispatches() {
        let mut app = app();
        let result = swarm(&mut app, Some("refactor the auth module to use V4 API"));
        assert!(app.swarm_active, "one-shot /swarm <task> must activate");
        assert_eq!(app.swarm_mode, SwarmMode::Standard);
        match result.action {
            Some(AppAction::SendMessage(ref text)) => {
                assert_eq!(text, "refactor the auth module to use V4 API");
            }
            _ => panic!("expected SendMessage action, got {:?}", result.action),
        }
    }

    #[test]
    fn swarm_big_one_shot_activates_big_mode_and_dispatches() {
        let mut app = app();
        let result = swarm_big(&mut app, Some("audit every crate for unwraps"));
        assert!(app.swarm_active, "one-shot /swarm-big <task> must activate");
        assert_eq!(app.swarm_mode, SwarmMode::Big);
        match result.action {
            Some(AppAction::SendMessage(ref text)) => {
                assert_eq!(text, "audit every crate for unwraps");
            }
            _ => panic!("expected SendMessage action, got {:?}", result.action),
        }
    }

    #[test]
    fn reserved_control_words_with_trailing_text_dispatch_as_tasks() {
        let mut app = app();
        let result = swarm(&mut app, Some("on fix the parser panic"));
        assert!(app.swarm_active);
        match result.action {
            Some(AppAction::SendMessage(ref text)) => {
                assert_eq!(text, "on fix the parser panic");
            }
            _ => panic!("expected SendMessage action, got {:?}", result.action),
        }

        let result = swarm_big(&mut app, Some("status audit every platform"));
        assert_eq!(app.swarm_mode, SwarmMode::Big);
        match result.action {
            Some(AppAction::SendMessage(ref text)) => {
                assert_eq!(text, "status audit every platform");
            }
            _ => panic!("expected SendMessage action, got {:?}", result.action),
        }
    }

    #[test]
    fn brief_sets_shows_and_clears() {
        let mut app = app();
        swarm(&mut app, Some("brief focus on the deepseek crate"));
        assert_eq!(
            app.swarm_brief.as_deref(),
            Some("focus on the deepseek crate")
        );

        let shown = swarm(&mut app, Some("brief"));
        assert!(
            shown
                .message
                .unwrap()
                .contains("focus on the deepseek crate")
        );

        swarm(&mut app, Some("brief clear"));
        assert!(app.swarm_brief.is_none());
    }

    #[test]
    fn status_reports_state_and_brief() {
        let mut app = app();
        let msg = swarm(&mut app, Some("status")).message.unwrap();
        assert!(msg.contains("OFF"));
        swarm(&mut app, Some("on"));
        swarm(&mut app, Some("brief focus area"));
        let msg = swarm(&mut app, Some("status")).message.unwrap();
        assert!(msg.contains("ON"));
        assert!(msg.contains("focus area"));
    }

    #[test]
    fn one_shot_keeps_swarm_on_for_followups() {
        // The user-facing workflow says "user prompts again -> repeat"
        // — so the one-shot dispatch must leave swarm_active = true so
        // the next plain submit also goes through the orchestrator.
        let mut app = app();
        swarm(&mut app, Some("kick off"));
        assert!(app.swarm_active);
    }
}
