//! Small helpers for the request-reply pattern against named actors.
//!
//! Every call-site otherwise repeats the same dance: look the actor up in the
//! registry, build a oneshot-backed [`RpcReplyPort`], send the message, then
//! await the receiver. [`query`] (for plain actors) and [`query_factory`] (for
//! actors behind a `ractor` factory) collapse that into a single call that takes
//! a closure building the message from the reply port.

use std::time::Duration;

use ractor::{
    RpcReplyPort,
    concurrency::oneshot,
    factory::{FactoryMessage, Job, JobOptions},
};

#[derive(thiserror::Error, Debug)]
pub enum RpcError {
    #[error("actor `{0}` not found")]
    ActorNotFound(&'static str),
    #[error("messaging error: {0}")]
    Messaging(String),
}

/// Send a request-reply message to a named actor and await the reply.
///
/// `make_msg` receives the reply port and returns the message to send, e.g.
/// `rpc::query(Foo::NAME, timeout, |reply| FooMsg::Query { id, reply })`.
pub async fn query<M, T>(
    name: &'static str,
    timeout: Duration,
    make_msg: impl FnOnce(RpcReplyPort<T>) -> M,
) -> Result<T, RpcError>
where
    M: ractor::Message,
    T: Send + 'static,
{
    let actor =
        ractor::registry::where_is(name.to_string()).ok_or(RpcError::ActorNotFound(name))?;

    let (tx, rx) = oneshot();
    let port: RpcReplyPort<T> = (tx, timeout).into();
    actor
        .send_message(make_msg(port))
        .map_err(|e| RpcError::Messaging(e.to_string()))?;

    rx.await.map_err(|e| RpcError::Messaging(e.to_string()))
}

/// Like [`query`], but for an actor behind a `ractor` factory: the message is
/// wrapped in a `FactoryMessage::Dispatch` job with the unit key.
pub async fn query_factory<M, T>(
    name: &'static str,
    timeout: Duration,
    make_msg: impl FnOnce(RpcReplyPort<T>) -> M,
) -> Result<T, RpcError>
where
    M: ractor::Message,
    T: Send + 'static,
{
    let actor =
        ractor::registry::where_is(name.to_string()).ok_or(RpcError::ActorNotFound(name))?;

    let (tx, rx) = oneshot();
    let port: RpcReplyPort<T> = (tx, timeout).into();
    let job = FactoryMessage::Dispatch(Job {
        key: (),
        msg: make_msg(port),
        options: JobOptions::default(),
        accepted: None,
    });
    actor
        .send_message(job)
        .map_err(|e| RpcError::Messaging(e.to_string()))?;

    rx.await.map_err(|e| RpcError::Messaging(e.to_string()))
}
