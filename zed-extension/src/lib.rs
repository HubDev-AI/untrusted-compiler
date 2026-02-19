use std::path::PathBuf;
use zed_extension_api as zed;

const LANGUAGE_SERVER_ID: &str = "sec4audit-lsp";
const LANGUAGE_SERVER_BINARY: &str = "sec4audit-language-server";

struct UntrustedExtension;

impl zed::Extension for UntrustedExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != LANGUAGE_SERVER_ID {
            return Err(format!("unsupported language server id: {language_server_id}").into());
        }

        let (configured_path, mut args, env) = read_lsp_binary_settings(worktree)?;
        let command = resolve_language_server_command(worktree, configured_path.as_deref())?;
        if !args.iter().any(|arg| arg == "--stdio") {
            args.push("--stdio".into());
        }

        Ok(zed::Command { command, args, env })
    }
}

fn read_lsp_binary_settings(
    worktree: &zed::Worktree,
) -> zed::Result<(Option<String>, Vec<String>, Vec<(String, String)>)> {
    let settings = zed::settings::LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)
        .map_err(|err| format!("could not read `{LANGUAGE_SERVER_ID}` settings: {err}"))?;
    let Some(binary) = settings.binary else {
        return Ok((None, Vec::new(), Vec::new()));
    };

    let path = binary.path.and_then(|path| {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let args = binary.arguments.unwrap_or_default();
    let mut env = binary
        .env
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    env.sort_by(|left, right| left.0.cmp(&right.0));
    Ok((path, args, env))
}

fn resolve_language_server_command(
    worktree: &zed::Worktree,
    configured_path: Option<&str>,
) -> zed::Result<String> {
    if let Some(path) = configured_path {
        if let Some(resolved) = worktree.which(path) {
            return Ok(resolved);
        }
        return Err(format!(
            "configured `{LANGUAGE_SERVER_ID}` binary path `{path}` was not found. \
Set a valid `lsp.{LANGUAGE_SERVER_ID}.binary.path` value or build `{LANGUAGE_SERVER_BINARY}` with `cargo build -p sec4audit-language-server`."
        )
        .into());
    }

    if let Some(command) = worktree.which(LANGUAGE_SERVER_BINARY) {
        return Ok(command);
    }

    let candidates = local_dev_binary_candidates(worktree.root_path());
    for candidate in &candidates {
        if let Some(command) = worktree.which(candidate) {
            return Ok(command);
        }
    }

    let searched = candidates
        .iter()
        .map(|candidate| format!("`{candidate}`"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "could not find `{LANGUAGE_SERVER_BINARY}`. Searched PATH and local fallbacks: {searched}. \
Build locally with `cargo build -p sec4audit-language-server` from `{}` or configure `lsp.{LANGUAGE_SERVER_ID}.binary.path`.",
        worktree.root_path()
    )
    .into())
}

fn local_dev_binary_candidates(worktree_root: String) -> Vec<String> {
    let binary = platform_binary_name();
    let mut candidates = Vec::new();

    push_candidate(
        &mut candidates,
        &worktree_root,
        &["target", "debug", binary],
    );
    push_candidate(
        &mut candidates,
        &worktree_root,
        &["target", "release", binary],
    );
    push_candidate(
        &mut candidates,
        &worktree_root,
        &["compiler", "sec4-lsp", "target", "debug", binary],
    );
    push_candidate(
        &mut candidates,
        &worktree_root,
        &["compiler", "sec4-lsp", "target", "release", binary],
    );

    candidates
}

fn push_candidate(candidates: &mut Vec<String>, root: &str, segments: &[&str]) {
    let mut path = PathBuf::from(root);
    for segment in segments {
        path.push(segment);
    }
    let candidate = path.to_string_lossy().into_owned();
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn platform_binary_name() -> &'static str {
    let (os, _) = zed::current_platform();
    match os {
        zed::Os::Windows => "sec4audit-language-server.exe",
        _ => LANGUAGE_SERVER_BINARY,
    }
}

zed::register_extension!(UntrustedExtension);
