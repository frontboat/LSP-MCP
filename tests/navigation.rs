use anyhow::{Context, Result, ensure};
use rmcp::{
    ClientLifecycleMode, ClientServiceExt, Peer, RoleClient,
    model::{CallToolRequestParams, ClientConfig, JsonObject, ProtocolVersion},
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::{fs, process::Command};

async fn call(client: &Peer<RoleClient>, tool: &'static str, args: Value) -> Result<Value> {
    let args: JsonObject = serde_json::from_value(args)?;
    let response = client
        .call_tool(CallToolRequestParams::new(tool).with_arguments(args))
        .await?;
    ensure!(
        response.is_error != Some(true),
        "{tool}: {:?}",
        response.content
    );
    response
        .structured_content
        .context("missing structured content")
}

#[tokio::test]
async fn navigate_and_refresh_saved_sources_over_stdio() -> Result<()> {
    let tmp = tempdir()?;
    let root = fs::canonicalize(tmp.path()).await?;
    let source = root.join("lib #1.ts");
    let caller = root.join("main.ts");
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true},"include":["*.ts"]}"#,
    )
    .await?;
    fs::write(&source, "export const value = 3;\r\n").await?;
    fs::write(
        &caller,
        "import { value } from './lib #1';\r\n/*😀*/ value;\r\n",
    )
    .await?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_lsp-mcp"));
    command.arg("--workspace").arg(&root);
    let client = ClientConfig::default()
        .serve_with_lifecycle(
            TokioChildProcess::new(command)?,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await?;
    let manifest: Value =
        serde_json::from_str(include_str!("../plugins/typescript-lsp/plugin.json"))?;
    let peer = client.peer_info().context("missing discovery result")?;
    let identity = peer
        .server_info
        .as_ref()
        .context("missing server identity")?;
    let display_name = &manifest["extensions"]["com.openai"]["interface"]["displayName"];
    assert_eq!(display_name, &identity.name);
    assert_eq!(identity.title.as_deref(), display_name.as_str());
    assert_eq!(manifest["version"], identity.version);
    assert_eq!(
        identity.description.as_deref(),
        Some(manifest["description"].as_str().unwrap())
    );
    let instructions = peer
        .instructions
        .as_deref()
        .context("missing tool instructions")?;
    let tools = client.list_all_tools().await?;
    assert_eq!(tools.len(), 3);
    for tool in &tools {
        assert!(
            instructions.contains(tool.name.as_ref()),
            "missing instructions for {}",
            tool.name
        );
    }
    assert!(
        tools
            .iter()
            .all(|tool| tool.annotations.as_ref().and_then(|a| a.read_only_hint) == Some(true))
    );

    let usage = json!({"file":"main.ts","line":2,"character":8});
    let expected = json!({"file":source,"line":1,"character":14,"end_line":1,"end_character":19});
    assert_eq!(
        call(&client, "go_to_definition", usage.clone()).await?,
        json!({"locations":[expected]})
    );
    let origin = json!({"file":"lib #1.ts","line":1,"character":14});
    let references = call(&client, "find_references", origin.clone()).await?;
    let locations = references["locations"]
        .as_array()
        .context("locations missing")?;
    assert!(locations.contains(&expected));
    assert!(
        locations.contains(
            &json!({"file":caller,"line":2,"character":8,"end_line":2,"end_character":13})
        )
    );
    let hover = call(&client, "hover", usage.clone()).await?;
    assert!(hover["text"].as_str().unwrap().contains("3"), "{hover}");

    fs::write(&source, "// moved\r\nexport const value = 'fresh';\r\n").await?;
    let moved = call(&client, "go_to_definition", usage.clone()).await?;
    assert_eq!(moved["locations"][0]["line"], 2);
    let hover = call(&client, "hover", usage).await?;
    assert!(hover["text"].as_str().unwrap().contains("fresh"), "{hover}");

    let error = call(
        &client,
        "hover",
        json!({"file":"main.ts","line":2,"character":4}),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("surrogate pair"), "{error}");
    client.cancel().await?;
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn sigterm_exits_while_client_keeps_stdin_open() -> Result<()> {
    use std::{process::Stdio, time::Duration};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut bridge = Command::new(env!("CARGO_BIN_EXE_lsp-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = bridge.stdin.take().unwrap();
    let mut stdout = BufReader::new(bridge.stdout.take().unwrap()).lines();
    let init = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"shutdown-test","version":"1"}
    }});
    stdin.write_all(format!("{init}\n").as_bytes()).await?;
    let message: Value =
        serde_json::from_str(&stdout.next_line().await?.context("unexpected EOF")?)?;
    assert_eq!(message["result"]["protocolVersion"], "2025-11-25");
    let signal = Command::new("kill")
        .args(["-TERM", &bridge.id().unwrap().to_string()])
        .status()
        .await?;
    assert!(signal.success());
    let status = tokio::time::timeout(Duration::from_secs(5), bridge.wait()).await??;
    assert!(status.success());
    drop(stdin);
    Ok(())
}

#[tokio::test]
async fn rust_analyzer_resolves_across_modules_on_the_first_lookup() -> Result<()> {
    let tmp = tempdir()?;
    let root = fs::canonicalize(tmp.path()).await?;
    fs::create_dir(root.join("src")).await?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .await?;
    let caller = root.join("src/lib.rs");
    let source = root.join("src/values.rs");
    fs::write(
        &caller,
        "mod values;\npub fn run() -> i32 {\n    /*😀*/ values::answer()\n}\n",
    )
    .await?;
    fs::write(&source, "/// The answer.\npub fn answer() -> i32 { 42 }\n").await?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_lsp-mcp"));
    command.args(["--language", "rust"]);
    let client = ClientConfig::default()
        .serve_with_lifecycle(
            TokioChildProcess::new(command)?,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await?;
    let peer = client
        .peer_info()
        .context("missing Rust discovery result")?;
    let info = peer.server_info.as_ref().context("missing Rust identity")?;
    let manifest: Value = serde_json::from_str(include_str!("../plugins/rust-lsp/plugin.json"))?;
    assert_eq!(info.name, "Rust LSP");
    assert_eq!(info.version, manifest["version"]);
    assert_eq!(
        info.description.as_deref(),
        manifest["description"].as_str()
    );
    let instructions = peer
        .instructions
        .as_deref()
        .context("missing Rust instructions")?;
    assert!(instructions.contains("Rust"));
    let tools = client.list_all_tools().await?;
    assert_eq!(tools.len(), 3);
    for tool in tools {
        assert!(instructions.contains(tool.name.as_ref()));
        assert_eq!(tool.annotations.and_then(|a| a.read_only_hint), Some(true));
    }

    let usage = json!({"file": caller, "line": 3, "character": 20});
    let definition = json!({"file":source,"line":2,"character":8,"end_line":2,"end_character":14});
    assert_eq!(
        call(&client, "go_to_definition", usage.clone()).await?,
        json!({"locations":[definition]})
    );
    let refs = call(&client, "find_references", json!({"workspace":root, "file":"src/values.rs","line":2,"character":8,"include_declaration":false})).await?;
    assert_eq!(
        refs,
        json!({"locations":[{"file":caller,"line":3,"character":20,"end_line":3,"end_character":26}]})
    );
    let hover = call(&client, "hover", usage.clone()).await?;
    let text = hover["text"].as_str().context("missing hover")?;
    assert!(
        text.contains("i32") && text.contains("The answer."),
        "{hover}"
    );

    fs::write(
        &source,
        "\n/// An updated answer.\npub fn answer() -> i32 { 43 }\n",
    )
    .await?;
    let moved = call(&client, "go_to_definition", usage.clone()).await?;
    assert_eq!(moved["locations"][0]["line"], 3);
    let hover = call(&client, "hover", usage).await?;
    assert!(
        hover["text"]
            .as_str()
            .is_some_and(|s| s.contains("An updated answer.")),
        "{hover}"
    );
    client.cancel().await?;
    Ok(())
}
