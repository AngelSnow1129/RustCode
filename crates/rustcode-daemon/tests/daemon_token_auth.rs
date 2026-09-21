use std::time::Duration;

use serial_test::serial;

// 本文件两条用例都改写进程级 `RUSTCODE_HOME`，并发跑会互相踩：`set_var` 之后
// 对方的 `remove_var` 会让本测试的 daemon 把 token 文件落进别的临时目录，断言
// 随之假红。全局环境变量是进程共享的单例，故整文件序列化。
#[tokio::test]
#[serial]
async fn chat_requires_token_health_is_public() {
    let tmp = std::env::temp_dir().join(format!("rustcode_it_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::env::set_var("RUSTCODE_HOME", &tmp);

    let port = 18099u16;
    let tmp_for_spawn = tmp.clone();
    let handle = tokio::spawn(async move {
        rustcode_daemon::run_server(rustcode_daemon::ServerOpts {
            host: "127.0.0.1".into(),
            port,
            idle_timeout_secs: 0,
            startup_mode: rustcode_daemon::client_mode::ClientMode::Ide,
            webui_tokens: {
                let store = rustcode_daemon::auth_token::WebuiTokenStore::new();
                store.insert("it-token".to_string());
                Some(store)
            },
            working_dir_override: Some(tmp_for_spawn),
            quiet: true,
            prebound_listener: None,
            daemon_token_file: Some("it-token".to_string()),
            webui_no_auth: false,
        })
        .await
        .ok();
    });

    // wait for bind
    tokio::time::sleep(Duration::from_millis(800)).await;
    let base = format!("http://127.0.0.1:{port}");

    let health = reqwest::get(format!("{base}/health")).await.unwrap();
    assert_eq!(health.status(), 200, "/health must be public");

    // GET /models without token must 401
    let no_tok = reqwest::Client::new()
        .get(format!("{base}/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        no_tok.status(),
        401,
        "protected route without token must 401"
    );

    let with_tok = reqwest::Client::new()
        .get(format!("{base}/models"))
        .header("Authorization", "Bearer it-token")
        .send()
        .await
        .unwrap();
    assert_ne!(with_tok.status(), 401, "valid token must not 401");

    // token file written with 0600
    let tf = tmp.join(format!("daemon-{port}.json"));
    assert!(tf.exists(), "daemon token file must exist");

    handle.abort();
    std::env::remove_var("RUSTCODE_HOME");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// 免密模式（`--no-auth` / `webui_no_auth` 的最终落点）：`webui_no_auth=true`
/// 时 store 里那个 token 不再被校验，匿名请求必须放行。
///
/// 反向锁定也在这里：同一套路由在 `webui_no_auth=false` 下必须 401（见上一条测试），
/// 否则"免密"就变成了"鉴权整体失效"。
#[tokio::test]
#[serial]
async fn no_auth_mode_serves_protected_routes_without_token() {
    let tmp = std::env::temp_dir().join(format!("rustcode_it_noauth_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::env::set_var("RUSTCODE_HOME", &tmp);

    // `/models` reads the on-disk config; a bare temp HOME would answer 500 and
    // a weak `!= 401` assertion would happily accept that. Seed one so the
    // assertion below proves the route really served, not merely "not 401".
    rustcode_config::config::Config::default()
        .save(&tmp.join("config.toml"))
        .expect("seed config for /models");

    let port = 18098u16;
    let tmp_for_spawn = tmp.clone();
    let handle = tokio::spawn(async move {
        rustcode_daemon::run_server(rustcode_daemon::ServerOpts {
            host: "127.0.0.1".into(),
            port,
            idle_timeout_secs: 0,
            startup_mode: rustcode_daemon::client_mode::ClientMode::Webui,
            webui_tokens: {
                let store = rustcode_daemon::auth_token::WebuiTokenStore::new();
                store.insert("it-token".to_string());
                Some(store)
            },
            webui_no_auth: true,
            working_dir_override: Some(tmp_for_spawn),
            quiet: true,
            prebound_listener: None,
            daemon_token_file: None,
        })
        .await
        .ok();
    });

    tokio::time::sleep(Duration::from_millis(800)).await;
    let base = format!("http://127.0.0.1:{port}");

    // 关键断言：不带任何凭证也必须放行（免密的定义）。
    //
    // 断言 200 而不是 `!= 401`：后者会被 404/500 一并满足，"鉴权失效"和
    // "免密生效"看起来一样绿。这里同时校验响应体是有效 JSON 数组，证明路由
    // 确实被服务了，而不是被静默跳过。
    let anon = reqwest::Client::new()
        .get(format!("{base}/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        anon.status(),
        200,
        "no-auth mode must serve protected routes anonymously"
    );
    let body = anon.text().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("/models returned non-JSON ({e}): {body}"));
    assert!(
        parsed.is_array(),
        "/models must answer with the model list, got: {body}"
    );

    handle.abort();
    std::env::remove_var("RUSTCODE_HOME");
    let _ = std::fs::remove_dir_all(&tmp);
}
