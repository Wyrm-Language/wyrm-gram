//! Gram/Zed extension for Wyrm: launches `wyrm-lsp` and `wyrm-dap` (both from
//! the `wypoc` distribution) and describes the debug configuration.
//!
//! Syntax highlighting, indentation and outline live in `languages/wyrm` and
//! the tree-sitter grammar in the tree-sitter-wyrm repository; this crate is only the
//! process-spawning half.

use std::collections::HashMap;

use zed_extension_api::{
    self as zed,
    serde_json::{self, json, Value},
    settings::LspSettings,
    DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario, DebugTaskDefinition,
    LanguageServerId, Result, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, TcpArgumentsTemplate, Worktree,
};

const LSP_NAME: &str = "wyrm-lsp";
const LSP_EXE: &str = "wyrm-lsp";
const DAP_EXE: &str = "wyrm-dap";

struct WyrmExtension;

/// Where an executable was found, best guess first: the user's explicit
/// setting, the workspace's own `.venv` (the layout `wypoc` documents for
/// development), then `$PATH`.
fn find_executable(
    worktree: &Worktree,
    exe: &str,
    configured: Option<String>,
    setting_hint: &str,
) -> Result<String> {
    if let Some(path) = configured.filter(|p| !p.trim().is_empty()) {
        return Ok(path);
    }

    // An extension can only read files through the worktree, and only as text.
    // Console-script launchers written by pip are text, which is enough to
    // tell whether the file exists.
    let venv = format!(".venv/bin/{exe}");
    if worktree.read_text_file(&venv).is_ok() {
        return Ok(format!("{}/{venv}", worktree.root_path()));
    }

    worktree.which(exe).ok_or_else(|| {
        format!(
            "`{exe}` was not found in {}/.venv/bin or on PATH. Install wypoc \
             (`pip install -e \".[lsp]\"`) or set {setting_hint}.",
            worktree.root_path()
        )
    })
}

/// Module search path for `import`: the `module_path` array from the
/// language server's `settings` (relative entries and `${worktree}` resolve
/// against the project root), followed by any `WYRM_PATH` already in the
/// environment, exactly as `wyrm.modulePath` does in the VS Code extension.
fn module_path(worktree: &Worktree, settings: Option<&Value>, ambient: Option<&str>) -> Option<String> {
    let root = worktree.root_path();
    let mut entries: Vec<String> = settings
        .and_then(|s| s.get("module_path"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(|entry| entry.trim())
                .filter(|entry| !entry.is_empty())
                .map(|entry| {
                    let entry = entry.replace("${worktree}", &root);
                    if entry.starts_with('/') {
                        entry
                    } else {
                        format!("{root}/{entry}")
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let inherit = settings
        .and_then(|s| s.get("module_path_inherit_environment"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if inherit {
        if let Some(ambient) = ambient.filter(|a| !a.is_empty()) {
            entries.push(ambient.to_string());
        }
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries.join(":"))
    }
}

/// The environment both wyrm tools run in: the shell's, then `WYRM_PATH`
/// from settings, then anything set explicitly in `binary.env`.
fn environment(worktree: &Worktree, settings: &LspSettings) -> Vec<(String, String)> {
    let mut env: HashMap<String, String> = worktree.shell_env().into_iter().collect();

    let ambient = env.get("WYRM_PATH").cloned();
    if let Some(path) = module_path(worktree, settings.settings.as_ref(), ambient.as_deref()) {
        env.insert("WYRM_PATH".into(), path);
    }
    if let Some(explicit) = settings.binary.as_ref().and_then(|b| b.env.as_ref()) {
        env.extend(explicit.clone());
    }

    env.into_iter().collect()
}

impl zed::Extension for WyrmExtension {
    fn new() -> Self {
        WyrmExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(LSP_NAME, worktree).unwrap_or_default();
        let binary = settings.binary.as_ref();

        let command = find_executable(
            worktree,
            LSP_EXE,
            binary.and_then(|b| b.path.clone()),
            "`lsp.wyrm-lsp.binary.path`",
        )?;

        Ok(zed::Command {
            command,
            args: binary.and_then(|b| b.arguments.clone()).unwrap_or_default(),
            env: environment(worktree, &settings),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(LspSettings::for_worktree(LSP_NAME, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(LspSettings::for_worktree(LSP_NAME, worktree)
            .ok()
            .and_then(|s| s.settings))
    }

    // ---- debugging ---------------------------------------------------------

    fn get_dap_binary(
        &mut self,
        _adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        // The adapter shares the language server's environment (WYRM_PATH), so
        // `import`s resolve the same way under the debugger as in the editor.
        let settings = LspSettings::for_worktree(LSP_NAME, worktree).unwrap_or_default();

        let command = find_executable(
            worktree,
            DAP_EXE,
            user_provided_debug_adapter_path,
            "`dap.wyrm.binary` in your debug settings",
        )?;

        let request = self.dap_request_kind(
            "wyrm".to_string(),
            serde_json::from_str(&config.config).map_err(|e| e.to_string())?,
        )?;

        Ok(DebugAdapterBinary {
            command: Some(command),
            arguments: vec![],
            envs: environment(worktree, &settings),
            cwd: Some(worktree.root_path()),
            // wyrm-dap speaks DAP over stdio.
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration: config.config,
                request,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        match config.get("request").and_then(Value::as_str) {
            None | Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some("attach") => Err("wyrm-dap does not support `attach`; use `launch`".into()),
            Some(other) => Err(format!("unknown request `{other}`; expected `launch`")),
        }
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        let DebugRequest::Launch(launch) = config.request else {
            return Err("wyrm-dap does not support `attach`; use `launch`".into());
        };

        let mut configuration = json!({
            "request": "launch",
            "program": launch.program,
            "args": launch.args,
            "stopOnEntry": config.stop_on_entry.unwrap_or(false),
        });
        if let Some(cwd) = launch.cwd {
            configuration["cwd"] = json!(cwd);
        }

        Ok(DebugScenario {
            label: config.label,
            adapter: config.adapter,
            build: None,
            config: configuration.to_string(),
            tcp_connection: None::<TcpArgumentsTemplate>,
        })
    }
}

zed::register_extension!(WyrmExtension);
