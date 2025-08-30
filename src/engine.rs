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
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{
        Implementation, InitializeRequestParam, InitializeResult,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
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
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            inner: artifex_engine::Engine::default(),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
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
}

#[tool_handler]
impl ServerHandler for Engine {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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
