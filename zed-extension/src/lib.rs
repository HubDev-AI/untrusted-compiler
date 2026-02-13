use zed_extension_api as zed;

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
        if language_server_id.as_ref() != "sec4audit-lsp" {
            return Err(format!("unsupported language server id: {language_server_id}").into());
        }

        let command = worktree
            .which("sec4audit-language-server")
            .unwrap_or_else(|| "sec4audit-language-server".into());

        Ok(zed::Command {
            command,
            args: vec!["--stdio".into()],
            env: vec![],
        })
    }
}

zed::register_extension!(UntrustedExtension);
