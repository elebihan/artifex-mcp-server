//
// This file is part of artifex-mcp-server
//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

mod cli;
mod config;

use artifex_mcp_server_service::Engine;
use cli::{TransportMode, build_cli};
use config::Config;
use rmcp::{
    ServiceExt,
    transport::{
        stdio,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService,
            session::local::LocalSessionManager,
        },
    },
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Starting Artifex MCP Server");
    let cli = build_cli();
    let config = if let Some(path) = cli.config {
        Config::with_path(path)?
    } else {
        Config::default()
    };
    let engine_config = config.engine.clone();
    match cli.transport {
        TransportMode::Http => {
            let service = StreamableHttpService::new(
                move || Ok(Engine::with_config(engine_config.clone())),
                LocalSessionManager::default().into(),
                StreamableHttpServerConfig::default(),
            );
            let router = axum::Router::new().nest_service("/mcp", service);
            let listener =
                tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    tokio::signal::ctrl_c().await.unwrap()
                })
                .await;
        }
        TransportMode::Stdio => {
            let service = Engine::with_config(engine_config)
                .serve(stdio())
                .await
                .inspect_err(|e| {
                    tracing::error!("Failed to serve: {:?}", e);
                })?;
            service.waiting().await?;
        }
    }
    Ok(())
}
