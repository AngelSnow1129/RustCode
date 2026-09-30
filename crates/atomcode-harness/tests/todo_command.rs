//! `todo` — a person's hand on the plan (`plugins::policy_rows::TodoCommand`).
//!
//! An edit is a `todowrite` call and its result, committed between turns: the
//! same facts the model writes, so everything that reads the plan reads it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use atomcode_capabilities::tools::todo::{derive_current_todos, TodoStatus};
use atomcode_harness::agent::Agent;
use atomcode_harness::session::{derive_messages, RewindScope, SessionEvent};
use atomcode_harness::{bundle, plugins};
use atomcode_plexus::{App, ConfigTree, Layer};

fn scratch(tag: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("todo-command-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn tree(dir: &Path) -> ConfigTree {
    let root = bundle::toml_string(&dir.to_string_lossy());
    let base = format!(
        "[[insert]]\nname = \"tool-fs-world\"\n\n\
         [[insert]]\nname = \"tool-todo\"\n\n\
         [[patch]]\nid = \"fs\"\nconfig = {{ root = {root} }}\n\n\
         [[patch]]\nid = \"llm\"\nname = \"llm-replay\"\nconfig = {{ script = [{{ text = \"ok\" }}] }}\n"
    );
    ConfigTree::from_layers([
        bundle::infra().expect("infra"),
        Layer::from_toml(&base).expect("rows"),
    ])
    .expect("tree")
}

async fn started(tag: &str) -> (App, Arc<Agent>) {
    let dir = scratch(tag);
    let mut app = App::new(plugins::catalog(), tree(&dir));
    app.start().await.expect("must mount");
    let agent = atomcode_harness::create_agent(&app)
        .await
        .expect("an agent to run commands against");
    (app, agent)
}

async fn todo(app: &App, agent: &Arc<Agent>, args: &str) -> Result<String, String> {
    app.context()
        .service::<atomcode_harness::seams::CommandsSvc>()
        .expect("the catalog is a core row")
        .find("todo", agent)
        .expect("`todo` is on offer where `todowrite` is")
        .run(agent.clone(), args)
        .await
}

/// What the model would be told the plan is, from the log alone.
fn plan(agent: &Agent) -> Vec<(String, TodoStatus)> {
    derive_current_todos(&derive_messages(&agent.session().events()))
        .into_iter()
        .map(|t| (t.content, t.status))
        .collect()
}

/// `add` appends a pending item, written as the model writes one — a
/// `todowrite` call and its result — so the list the model is told about is
/// the list the person sees; `clear` empties it.
#[tokio::test]
async fn a_person_adds_to_the_plan_and_clears_it() {
    let (app, agent) = started("add").await;

    assert_eq!(
        todo(&app, &agent, "add 补上回归测试").await,
        Ok("Added task: 补上回归测试".into())
    );
    todo(&app, &agent, "add 跑一遍全量")
        .await
        .expect("second add");
    assert_eq!(
        plan(&agent),
        [
            ("补上回归测试".to_string(), TodoStatus::Pending),
            ("跑一遍全量".to_string(), TodoStatus::Pending),
        ]
    );
    let events = agent.session().events();
    let calls = events
        .iter()
        .filter(|e| {
            matches!(&e.event, SessionEvent::AssistantMessage { tool_calls, .. }
                if tool_calls.iter().any(|c| c.name == "todowrite"))
        })
        .count();
    let results = events
        .iter()
        .filter(|e| matches!(e.event, SessionEvent::ToolResultLogged { .. }))
        .count();
    assert_eq!(
        (calls, results),
        (2, 2),
        "each edit is one call and its result"
    );

    let listed = todo(&app, &agent, "").await.expect("listing");
    assert!(
        listed.contains("补上回归测试") && listed.contains("跑一遍全量"),
        "{listed}"
    );

    todo(&app, &agent, "clear").await.expect("clear");
    assert!(plan(&agent).is_empty(), "cleared: {:?}", plan(&agent));

    assert!(
        todo(&app, &agent, "add").await.is_err(),
        "`add` with nothing to add"
    );
    assert!(
        todo(&app, &agent, "remove 1").await.is_err(),
        "and a word it does not know"
    );
}

/// Inside a turn the pair would land between a call and its result, which is
/// a request the provider rejects: refused, and nothing written.
#[tokio::test]
async fn an_edit_waits_for_the_turn_to_end() {
    let (app, agent) = started("busy").await;
    agent.begin_turn();
    let before = agent.session().events().len();
    assert!(todo(&app, &agent, "add 不该进去").await.is_err());
    assert_eq!(
        agent.session().events().len(),
        before,
        "nothing was written"
    );
    agent.end_turn();
    todo(&app, &agent, "add 这回可以")
        .await
        .expect("between turns");
    assert_eq!(plan(&agent).len(), 1);
}

/// Filed under a turn that was taken back, the edit would be hidden with that
/// turn on screen while the model still read it. It gets a turn of its own.
#[tokio::test]
async fn an_edit_after_an_undo_is_not_filed_under_the_undone_turn() {
    let (app, agent) = started("undo").await;
    let log = agent.session();
    let turn = log.next_turn();
    let start = log.append(SessionEvent::TurnStart { turn });
    log.append(SessionEvent::Rewound {
        turn,
        to: start,
        scope: RewindScope::Conversation,
    });
    let undone = atomcode_kernel::session::undone_turns(&log.events());
    assert!(undone.contains(&turn), "the setup undid turn {turn}");

    todo(&app, &agent, "add 撤回之后加的").await.expect("add");
    let filed: Vec<u64> = log
        .events()
        .iter()
        .filter_map(|e| match &e.event {
            SessionEvent::ToolResultLogged { turn, .. } => Some(*turn),
            _ => None,
        })
        .collect();
    assert_eq!(filed.len(), 1);
    assert!(
        !undone.contains(&filed[0]),
        "filed under undone turn {}",
        filed[0]
    );
    assert_eq!(plan(&agent).len(), 1, "and the model reads it");
}
