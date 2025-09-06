//
// This file is part of artifex-mcp-server
//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_engine::{self, Config, MachineInfo, ProgramOutput};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::{Json, Parameters},
    },
    model::{
        GetPromptRequestParam, GetPromptResult, Implementation,
        InitializeRequestParam, InitializeResult, ListPromptsResult,
        PaginatedRequestParam, PromptMessage, PromptMessageRole,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    prompt, prompt_handler, prompt_router,
    schemars::{self, JsonSchema},
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result from an inspection.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InspectResult {
    /// Version of the kernel.
    pub kernel_version: String,
    /// Time elapsed since boot.
    pub system_uptime: Duration,
}

impl From<MachineInfo> for InspectResult {
    fn from(value: MachineInfo) -> Self {
        Self {
            kernel_version: value.kernel_version,
            system_uptime: value.system_uptime,
        }
    }
}

/// Request for command execution.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteRequest {
    /// Command to execute.
    command: String,
    /// Arguments for the command.
    arguments: Vec<String>,
}

/// Result from a command execution.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteResult {
    /// Command exit code.
    code: i32,
    /// Command standard output.
    stdout: String,
    /// Command standard error.
    stderr: String,
}

/// Result from an upgrade operation.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpgradeResult {
    /// Whether the upgrade completed successfully or not.
    completed: bool,
}

/// Arguments for the artifex_execute prompt.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecutePromptArgs {
    /// Program to execute.
    pub command: String,
    /// Optional arguments to pass to the program.
    #[serde(default)]
    pub arguments: Vec<String>,
}

impl From<ProgramOutput> for ExecuteResult {
    fn from(value: ProgramOutput) -> Self {
        Self {
            code: value.code,
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

/// Represent the engine.
pub struct Engine {
    inner: artifex_engine::Engine,
    tool_router: ToolRouter<Engine>,
    prompt_router: PromptRouter<Engine>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            inner: artifex_engine::Engine::default(),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }
}

#[tool_router]
#[prompt_router]
impl Engine {
    /// Create a new engine.
    pub fn with_config(config: Config) -> Self {
        Self {
            inner: artifex_engine::Engine::with_config(config),
            ..Default::default()
        }
    }

    #[tool(description = "Execute a command on a system")]
    pub fn execute(
        &self,
        Parameters(ExecuteRequest { command, arguments }): Parameters<
            ExecuteRequest,
        >,
    ) -> Result<Json<ExecuteResult>, McpError> {
        let output = self
            .inner
            .execute(command, arguments)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Json(output.into()))
    }

    #[tool(description = "Inspect system")]
    pub fn inspect(&self) -> Result<Json<InspectResult>, McpError> {
        let info = self
            .inner
            .inspect()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Json(info.into()))
    }

    #[tool(description = "Upgrade engine (streaming progress)")]
    pub fn upgrade(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<UpgradeResult>, McpError> {
        let peer = context.peer.clone();
        let progress_token = context.meta.get_progress_token();
        self.inner
            .upgrade(|position| {
                if let Some(token) = progress_token.clone() {
                    let peer = peer.clone();
                    tokio::spawn(async move {
                        let _ = peer
                            .notify_progress(
                                rmcp::model::ProgressNotificationParam {
                                    progress_token: token,
                                    progress: position as f64,
                                    total: Some(100.0),
                                    message: Some(format!(
                                        "Upgrade: {position}%"
                                    )),
                                },
                            )
                            .await;
                    });
                }
            })
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Json(UpgradeResult { completed: true }))
    }

    #[prompt(
        description = "Template to inspect the host and summarize findings"
    )]
    pub fn artifex_inspect(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Use the 'inspect' tool to retrieve the kernel version and system uptime. Then summarize the results in 1–2 sentences.",
        )]
    }

    #[prompt(description = "Run a command via the execute tool")]
    pub fn artifex_execute(
        &self,
        Parameters(args): Parameters<ExecutePromptArgs>,
    ) -> Vec<PromptMessage> {
        let mut msg =
            format!("Use the 'execute' tool to run '{}'", args.command);
        if !args.arguments.is_empty() {
            msg.push_str(&format!(" with arguments {:?}", args.arguments));
        }
        msg.push_str(
            ". Capture stdout and stderr. Return a concise summary and include short outputs.",
        );
        vec![PromptMessage::new_text(PromptMessageRole::User, msg)]
    }

    #[prompt(
        description = "Start an engine upgrade and follow streamed progress"
    )]
    pub fn artifex_upgrade(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Call the 'upgrade' tool and wait for completion. The server streams progress updates; report final status concisely when done.",
        )]
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for Engine {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides Artifex Engine tools. Tools: inspect"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
