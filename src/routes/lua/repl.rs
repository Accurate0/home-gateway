use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{
    Auth, AuthContext,
    scope::{Action, Resource, Scope},
};
use crate::error::AppError;
use crate::lua::{LuaAuthority, LuaCallContext, LuaSession, Script};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LuaReplQuery {
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Deserialize)]
struct LuaReplRequest {
    script: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LuaReplReply {
    Result { result: serde_json::Value },
    Error { error: String },
}

pub async fn lua_repl(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Query(query): Query<LuaReplQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, AppError> {
    auth.require(&Scope::new(Resource::Lua, Action::Write))
        .map_err(AppError::StatusCode)?;

    Ok(upgrade.on_upgrade(move |socket| serve(state, auth, query.dry_run, socket)))
}

async fn serve(state: AppState, auth: AuthContext, dry_run: bool, mut socket: WebSocket) {
    let event_id = Uuid::new_v4();

    let cx = LuaCallContext::new(state.clone(), event_id, "repl")
        .with_dry_run(dry_run)
        .with_authority(LuaAuthority::delegated(auth));

    let session = match state.lua.session(cx) {
        Ok(session) => session,
        Err(e) => {
            tracing::error!("[{event_id}] failed to start a lua repl session: {e}");
            reply(
                &mut socket,
                LuaReplReply::Error {
                    error: e.to_string(),
                },
            )
            .await;
            return;
        }
    };

    tracing::info!(
        "[{event_id}] opened a lua repl session for {} (dry_run: {dry_run})",
        session.context().authority.describe()
    );

    while let Some(message) = socket.recv().await {
        let text = match message {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(other) => {
                tracing::debug!("[{event_id}] ignoring non-text repl frame: {other:?}");
                continue;
            }
            Err(e) => {
                tracing::warn!("[{event_id}] lua repl socket failed: {e}");
                break;
            }
        };

        let outcome = evaluate(&session, &text).await;

        if !reply(&mut socket, outcome).await {
            break;
        }
    }

    tracing::info!("[{event_id}] closed the lua repl session");
}

async fn evaluate(session: &LuaSession, text: &str) -> LuaReplReply {
    let request = match serde_json::from_str::<LuaReplRequest>(text) {
        Ok(request) => request,
        Err(e) => {
            return LuaReplReply::Error {
                error: format!("malformed request: {e}"),
            };
        }
    };

    let script = match Script::parse(&request.script) {
        Ok(script) => script,
        Err(error) => return LuaReplReply::Error { error },
    };

    match session.eval(&script).await {
        Ok(result) => LuaReplReply::Result { result },
        Err(e) => LuaReplReply::Error {
            error: e.to_string(),
        },
    }
}

async fn reply(socket: &mut WebSocket, reply: LuaReplReply) -> bool {
    let body = match serde_json::to_string(&reply) {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("failed to encode a lua repl reply: {e}");
            return false;
        }
    };

    socket.send(Message::Text(body.into())).await.is_ok()
}
