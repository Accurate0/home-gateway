use std::collections::BTreeMap;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use home_gateway::actors::devices::handler::spawn_handler;
use home_gateway::actors::devices::light::LightHandler;
use home_gateway::actors::system::rpc;
use home_gateway::actors::workflows::spawn::spawn_workflows;
use home_gateway::actors::workflows::{WorkflowWorker, WorkflowWorkerMessage};
use home_gateway::auth::AuthContext;
use home_gateway::lua::{Script, execute};
use home_gateway::settings::WorkflowDefinition;
use home_gateway::variables::{Node, Value, Vars};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{Harness, wait_for};

const DB_TIMEOUT: Duration = Duration::from_secs(10);
const LAMP_TOPIC: &str = "zigbee2mqtt/0x0000000000000003/set";
const SETTLE: Duration = Duration::from_millis(750);

async fn start() -> Harness {
    let harness = Harness::start().await;

    spawn_workflows(&harness.root, harness.state.clone())
        .await
        .expect("failed to spawn the workflow factory");
    spawn_handler::<LightHandler>(&harness.root, harness.state.clone())
        .await
        .expect("failed to spawn the light handler");

    harness
}

fn run_workflow(harness: &Harness, name: &str, input: Node) {
    let workflow = harness
        .state
        .settings
        .workflows
        .get(name)
        .map(WorkflowDefinition::body)
        .unwrap_or_else(|| panic!("the fixture config should declare `{name}`"))
        .clone();

    rpc::cast_factory(
        WorkflowWorker::NAME,
        WorkflowWorkerMessage::Execute {
            event_id: Uuid::new_v4(),
            workflow,
            vars: Vars::default().with("input", input),
            traceparent: None,
        },
    )
    .expect("failed to dispatch the workflow");
}

async fn mint_key(harness: &Harness, scopes: &[&str]) -> String {
    let key = format!("test-key-{}", Uuid::new_v4().simple());
    let hashed = home_gateway::auth::hash_key(&key);
    let scopes: Vec<String> = scopes.iter().map(|scope| (*scope).to_owned()).collect();

    sqlx::query(
        "INSERT INTO api_keys (name, key_prefix, key_hash, scopes) VALUES ($1, $2, $3, $4)",
    )
    .bind("lua-test")
    .bind(&key[..8])
    .bind(&hashed)
    .bind(&scopes)
    .execute(&harness.db)
    .await
    .expect("failed to insert the test api key");

    key
}

async fn runs_for(harness: &Harness, slug: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM workflow_runs WHERE slug = $1")
        .bind(slug)
        .fetch_one(&harness.db)
        .await
        .unwrap()
}

async fn error_for(harness: &Harness, slug: &str) -> Option<String> {
    wait_for(
        DB_TIMEOUT,
        &format!("a workflow run for {slug}"),
        || async {
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT error FROM workflow_runs WHERE slug = $1 ORDER BY started_at DESC LIMIT 1",
            )
            .bind(slug)
            .fetch_optional(&harness.db)
            .await
            .unwrap()
        },
    )
    .await
}

#[tokio::test]
#[serial]
async fn a_lua_step_drives_a_device_and_publishes_its_returns_to_later_steps() {
    let harness = start().await;

    run_workflow(&harness, "Test lua step", Node::empty());

    let lamp = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(lamp, serde_json::json!({ "state": "ON" }));

    let label = harness.recorder.expect_publish("test/lua/label").await;
    assert_eq!(label, serde_json::json!({ "label": "lit", "watts": 42 }));
}

#[tokio::test]
#[serial]
async fn a_lua_guard_gates_its_step() {
    let harness = start().await;

    let mut allowed = Node::empty();
    allowed.insert("go", Node::Value(Some(Value::Bool(true))));
    run_workflow(&harness, "Test lua guard", allowed);

    let passed = harness.recorder.expect_publish("test/lua/guard").await;
    assert_eq!(passed, serde_json::json!({ "passed": true }));
}

#[tokio::test]
#[serial]
async fn a_failing_lua_guard_skips_its_step() {
    let harness = start().await;

    let mut denied = Node::empty();
    denied.insert("go", Node::Value(Some(Value::Bool(false))));
    run_workflow(&harness, "Test lua guard", denied);

    wait_for(DB_TIMEOUT, "the guarded workflow to finish", || async {
        (runs_for(&harness, "test-lua-guard").await > 0).then_some(())
    })
    .await;

    harness.recorder.assert_no_publish("test/lua/guard");
}

#[tokio::test]
#[serial]
async fn a_dry_run_workflow_suppresses_every_lua_side_effect() {
    let harness = start().await;

    run_workflow(&harness, "Test lua dry run", Node::empty());

    wait_for(DB_TIMEOUT, "the dry-run workflow to finish", || async {
        (runs_for(&harness, "test-lua-dry-run").await > 0).then_some(())
    })
    .await;

    tokio::time::sleep(SETTLE).await;

    harness.recorder.assert_no_publish(LAMP_TOPIC);
    harness.recorder.assert_no_publish("test/lua/dry-run");
}

#[tokio::test]
#[serial]
async fn a_self_referential_lua_script_is_stopped_by_the_depth_guard() {
    let harness = start().await;

    run_workflow(&harness, "Test lua recursion", Node::empty());

    let error = error_for(&harness, "test-lua-recursion")
        .await
        .expect("the recursive workflow should have failed");

    assert!(
        error.contains("recursion depth exceeded"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
#[serial]
async fn the_sandbox_is_enforced_at_runtime() {
    let harness = start().await;

    let script = Script::parse("return io == nil and require == nil and os.execute == nil")
        .expect("the probe should compile");

    let result = execute::execute(
        &harness.state,
        AuthContext::full_access(false),
        &script,
        BTreeMap::new(),
        false,
    )
    .await
    .expect("the probe should run");

    assert_eq!(result, serde_json::json!(true));
}

#[tokio::test]
#[serial]
async fn an_infinite_loop_is_stopped_by_the_instruction_limit() {
    let harness = start().await;

    let script = Script::parse("while true do end").expect("the loop should compile");

    let error = execute::execute(
        &harness.state,
        AuthContext::full_access(false),
        &script,
        BTreeMap::new(),
        false,
    )
    .await
    .expect_err("the loop should be interrupted");

    assert!(
        error.to_string().contains("instructions"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
#[serial]
async fn a_delegated_script_only_sees_what_its_scopes_allow() {
    let harness = start().await;

    let auth = AuthContext::from_scopes(
        None,
        Some("reader".to_owned()),
        &["light:read".to_owned(), "solar:read".to_owned()],
    );

    let script = Script::parse(
        "return {
            light_is_on = light.is_on ~= nil,
            light_set = light.set ~= nil,
            solar = solar ~= nil,
            mqtt = mqtt ~= nil,
            notify = notify ~= nil,
            workflow = workflow ~= nil,
            http = gw.http ~= nil,
            log = gw.log ~= nil,
        }",
    )
    .expect("the probe should compile");

    let result = execute::execute(&harness.state, auth, &script, BTreeMap::new(), false)
        .await
        .expect("the probe should run");

    assert_eq!(
        result,
        serde_json::json!({
            "light_is_on": true,
            "light_set": false,
            "solar": true,
            "mqtt": false,
            "notify": false,
            "workflow": false,
            "http": false,
            "log": true,
        })
    );
}

#[tokio::test]
#[serial]
async fn a_split_scope_only_unlocks_its_own_functions() {
    let harness = start().await;

    let auth = AuthContext::from_scopes(
        None,
        Some("switcher".to_owned()),
        &["switch:write".to_owned(), "workflow:run".to_owned()],
    );

    let script = Script::parse(
        "return {
            switch_set = switch.set ~= nil,
            light = light ~= nil,
            mqtt = mqtt ~= nil,
            http = gw.http ~= nil,
            run = workflow.run ~= nil,
            set_mode = workflow.set_mode ~= nil,
        }",
    )
    .expect("the probe should compile");

    let result = execute::execute(&harness.state, auth, &script, BTreeMap::new(), false)
        .await
        .expect("the probe should run");

    assert_eq!(
        result,
        serde_json::json!({
            "switch_set": true,
            "light": false,
            "mqtt": false,
            "http": false,
            "run": true,
            "set_mode": false,
        })
    );
}

#[tokio::test]
#[serial]
async fn a_script_can_check_its_scopes_before_acting() {
    let harness = start().await;

    let auth =
        AuthContext::from_scopes(None, Some("reader".to_owned()), &["light:read".to_owned()]);

    let script =
        Script::parse("return { read = gw.has(\"light:read\"), write = gw.has(\"light:write\") }")
            .expect("the probe should compile");

    let result = execute::execute(
        &harness.state,
        auth.clone(),
        &script,
        BTreeMap::new(),
        false,
    )
    .await
    .expect("the probe should run");

    assert_eq!(result, serde_json::json!({ "read": true, "write": false }));

    let script = Script::parse("gw.require(\"light:write\")\nreturn true")
        .expect("the probe should compile");

    let error = execute::execute(
        &harness.state,
        auth.clone(),
        &script,
        BTreeMap::new(),
        false,
    )
    .await
    .expect_err("a missing scope should stop the script");

    assert!(
        error.to_string().contains("missing scope `light:write`"),
        "unexpected error: {error}"
    );

    let script =
        Script::parse("return gw.has(\"control:write\")").expect("the probe should compile");

    let error = execute::execute(&harness.state, auth, &script, BTreeMap::new(), false)
        .await
        .expect_err("an unknown scope should be rejected");

    assert!(
        error.to_string().contains("unknown resource"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
#[serial]
async fn a_delegated_script_cannot_drive_a_device_it_lacks_write_scope_for() {
    let harness = start().await;

    let auth =
        AuthContext::from_scopes(None, Some("reader".to_owned()), &["light:read".to_owned()]);
    let script = Script::parse("light.set(\"test-lamp\", { state = \"ON\" })")
        .expect("the probe should compile");

    let error = execute::execute(&harness.state, auth, &script, BTreeMap::new(), false)
        .await
        .expect_err("a read-only token should not reach light.set");

    assert!(
        error.to_string().contains("nil value"),
        "unexpected error: {error}"
    );

    harness.recorder.assert_no_publish(LAMP_TOPIC);
}

#[tokio::test]
#[serial]
async fn a_config_authored_script_keeps_full_access() {
    let harness = start().await;

    let script = home_gateway::lua::LuaSource::Script {
        script: Script::parse("return light.set ~= nil and mqtt ~= nil and workflow ~= nil")
            .expect("the probe should compile"),
    };

    let cx = home_gateway::lua::LuaCallContext::new(
        harness.state.clone(),
        Uuid::new_v4(),
        "workflow:test",
    );

    let result = harness
        .state
        .lua
        .run_bool(&cx, &script, &Vars::default())
        .await
        .expect("the probe should run");

    assert!(result, "a trusted script should see the whole api");
}

#[tokio::test]
#[serial]
async fn the_rest_route_executes_a_script_and_returns_its_value() {
    let harness = start().await;
    let router = harness.router();
    let key = mint_key(&harness, &["lua:write"]).await;

    let body = serde_json::json!({
        "script": "return { doubled = input.n * 2 }",
        "vars": { "input": { "n": 21 } },
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/lua/execute")
                .header("content-type", "application/json")
                .header("X-Api-Key", &key)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .expect("the router should not fail");

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(parsed["result"], serde_json::json!({ "doubled": 42 }));
}

#[tokio::test]
#[serial]
async fn the_graphql_mutation_executes_a_script() {
    let harness = start().await;
    let router = harness.router();
    let key = mint_key(&harness, &["lua:write"]).await;

    let body = serde_json::json!({
        "query": "mutation { executeLua(script: \"return 6 * 7\") }",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/graphql")
                .header("content-type", "application/json")
                .header("X-Api-Key", &key)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .expect("the router should not fail");

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(
        parsed["data"]["executeLua"],
        serde_json::json!(42),
        "unexpected graphql response: {parsed}"
    );
}

#[tokio::test]
#[serial]
async fn a_scripted_ingest_source_parses_its_payload() {
    let harness = start().await;
    let router = harness.router();
    let key = mint_key(&harness, &["ingest.lua:write"]).await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/lua/test-doorbell")
                .header("content-type", "application/json")
                .header("X-Api-Key", &key)
                .body(Body::from(r#"{"event": "pressed"}"#))
                .unwrap(),
        )
        .await
        .expect("the router should not fail");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let published = harness
        .recorder
        .expect_publish("test/ingest/doorbell")
        .await;
    assert_eq!(published, serde_json::json!({ "event": "pressed" }));
}

#[tokio::test]
#[serial]
async fn an_unknown_ingest_source_is_not_found() {
    let harness = start().await;
    let router = harness.router();
    let key = mint_key(&harness, &["ingest.lua:write"]).await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/lua/nope")
                .header("content-type", "application/json")
                .header("X-Api-Key", &key)
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .expect("the router should not fail");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
