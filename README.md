# Artifex MCP Server

⚠️ Disclaimer: this project is *FOR EDUCATION, NOT PRODUCTION** ️

This project is an example of [Model Context Protocol][MCP] server, which allows
a Large Language Model to interact with [Artifex Engine][artifex].

## Build instructions

To build the server, execute:

```sh
cargo build
```

## Usage

This MCP server can be used with [Ollama][ollama] via:

- [Ollama MCP Bridge][ollama-mcp-bridge], by using the "stdio" transport mode.
- [Ollama MCP Client][ollama-mcp-client], for a REPL interface.

### Set up ollama

Follow the [instructions][ollama] to install `ollama` on a GNU/Linux system.

Then, run Ollama:

``` sh
ollama serve
```

### Using ollama-mcp-bridge

Install `ollama-mcp-bridge` using [uv Python package manager][uv]:

``` sh
uv tool install ollama-mcp-bridge
```

Then, run the bridge:

``` sh
PATH=$PATH:${PWD}/target/debug ollama-mcp-bridge \
    --config ${PWD}/data/ollama-mcp-bridge/mcp-config.json \
    --host 0.0.0.0 --port 8000
```

Use `curl` to interact using the chat API. For example:

``` sh
curl -N -X POST http://localhost:8000/api/chat \
    -H "accept: application/json" \
    -H "Content-Type: application/json" \
    -d '{
    "model": "qwen3:0.6b",
    "messages": [
      {
        "role": "system",
        "content": "You are an Artifex Engine assistant."
      },
      {
        "role": "user",
        "content": "Give me the result of the inspection."
      }
    ],
    "think": false,
    "stream": false
  }'
```
### Using Ollama MCP Client

🔧 The configuration file ``data/ollama-mcp-bridge/mcp-config.json`` must be
modified.

Change:

```json
                "../../data/samples/config.toml"
```

To:

```json
                "./data/samples/config.toml"
```

Install `ollama-mcp-client` using `uv`:

``` sh
uv tool install ollmcp
```

Then, start the client:

``` sh
PATH=$PATH:${PWD}/target/debug ollmcp --model qwen3:0.6b \
    --servers-json ${PWD}/data/ollama-mcp-bridge/mcp-config.json
```

Use the `tools` command to list the available tools: "artifex-engine" should be
listed. Enable it and start chatting.

# License

Copyright © 2025 Eric Le Bihan

This program is distributed under the terms of the MIT License.

See the [LICENSE-MIT](LICENSE-MIT) file for license details.

[artifex]: https://github.com/elebihan/artifex
[MCP]: https://modelcontextprotocol.io/docs/getting-started/intro
[ollama]: https://ollama.com/download/linux
[ollama-mcp-bridge]: https://github.com/jonigl/ollama-mcp-bridge
[uv]: https://docs.astral.sh/uv/
