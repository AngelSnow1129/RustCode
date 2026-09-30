//! The review specialization's rows, `tool-code-review` and `persona-review`.
//!
//! They live in this crate rather than in the harness: the mechanism offers
//! `/review` over whatever reviewer tool a row hands it
//! (`plugins::capabilities::register_review_command`) and does not depend on
//! `atomcode-review`. What is judged here is that the rows still register what
//! they own, mounted on a plain harness tree plus this crate's rows.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use atomcode_harness::seams::{CommandsSvc, SystemPromptSvc};
use atomcode_harness::{bundle, plugins};
use atomcode_plexus::{App, ConfigTree, Layer};

#[ctor::ctor]
fn _isolate_atomcode_home() {
    atomcode_kernel::test_support::isolate_home();
}

fn scratch(tag: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("review-rows-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn tree(root: &std::path::Path, extra: &str) -> ConfigTree {
    let rows = format!(
        "[[patch]]\nid = \"trace\"\nconfig = {{ stream = false, tools = false, summary = false }}\n\n\
         [[patch]]\nid = \"fs\"\nconfig = {{ root = {root:?} }}\n\n\
         [[patch]]\nid = \"agent-loop\"\nconfig = {{ max_rounds = 6, working_dir = {root:?} }}\n\n\
         [[patch]]\nid = \"llm\"\nname = \"llm-replay\"\nconfig = {{ script = [ {{ text = \"ok\" }} ] }}\n",
        root = root.to_string_lossy(),
    );
    ConfigTree::from_layers(vec![
        bundle::base().unwrap(),
        Layer::from_toml(&rows).unwrap(),
        Layer::from_toml(extra).unwrap(),
    ])
    .unwrap()
}

async fn start(tree: ConfigTree) -> App {
    let mut catalog = plugins::catalog();
    for row in atomcode_coding::on_harness::plugins() {
        catalog.register(row);
    }
    let mut app = App::new(catalog, tree);
    app.start().await.expect("must mount");
    app
}

/// The reviewer row offers `/review`, and the argument its usage line
/// advertises actually parses.
///
/// `/review` takes `[staged | <base>]`, and both forms used to be rejected
/// outright: the command wrote the word under `scope`, an internally tagged
/// enum (`#[serde(tag = "kind")]`) that cannot take a bare string. Judged by
/// what the tool says back — the scratch directory is not a repository, so the
/// run stops at `git diff` and costs no model round, which is exactly the point
/// past the parse.
#[tokio::test]
async fn the_review_row_offers_review_and_its_arguments_parse() {
    let dir = scratch("review");
    let reviewing = start(tree(&dir, "[[insert]]\nname = \"tool-code-review\"\n")).await;
    let agent = atomcode_harness::create_agent(&reviewing)
        .await
        .expect("an agent");
    let catalog = reviewing
        .context()
        .service::<CommandsSvc>()
        .expect("the catalog");
    let offered: Vec<String> = catalog
        .offered_for(&agent)
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert!(
        offered.contains(&"review".to_string()),
        "the review row offers `/review`: {offered:?}"
    );
    for scope in ["staged", "main"] {
        let said = catalog
            .find("review", &agent)
            .expect("offered, so findable")
            .run(agent.clone(), scope)
            .await
            .err()
            .unwrap_or_default();
        assert!(
            !said.contains("invalid arguments") && !said.contains("invalid scope"),
            "`/review {scope}` got past the argument parser: {said}"
        );
    }
}

/// The reviewer prompt comes from the review specialization, through its row.
#[tokio::test]
async fn the_review_persona_row_contributes_the_reviewer_prompt() {
    let dir = scratch("persona");
    let app = start(tree(
        &dir,
        "[[patch]]\nid = \"persona-coding\"\ndisabled = true\n\n[[insert]]\nname = \"persona-review\"\n",
    ))
    .await;
    let prompt = app.context().service::<SystemPromptSvc>().unwrap().render();
    let expected = atomcode_review::review_persona("replay");
    let first_line = expected.lines().next().unwrap();
    assert!(
        prompt.contains(first_line),
        "the reviewer prompt is not in the tree's prompt:\n{prompt}"
    );
}
