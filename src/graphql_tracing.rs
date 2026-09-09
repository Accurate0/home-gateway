use async_graphql::{
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextValidation, ResolveInfo,
    },
    parser::types::{ExecutableDocument, Selection},
};
use futures_util::TryFutureExt;
use std::sync::Arc;
use tracing_futures::Instrument;

pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension)
    }
}

struct TracingExtension;

#[async_trait::async_trait]
impl Extension for TracingExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx)
            .instrument(tracing::span!(
                target: "async_graphql::graphql",
                tracing::Level::DEBUG,
                "request",
            ))
            .await
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
            "parse_query",
            source = tracing::field::Empty,
            operation = tracing::field::Empty,
            fields = tracing::field::Empty,
        );
        async move {
            let res = next.run(ctx, query, variables).await;
            if let Ok(doc) = &res {
                let current = tracing::Span::current();
                current.record("source", ctx.stringify_execute_doc(doc, variables).as_str());

                if let Some((name, operation)) = doc.operations.iter().next() {
                    if let Some(name) = name {
                        current.record("operation", name.as_str());
                    }

                    let fields = operation
                        .node
                        .selection_set
                        .node
                        .items
                        .iter()
                        .filter_map(|item| match &item.node {
                            Selection::Field(field) => Some(field.node.name.node.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(",");

                    current.record("fields", fields.as_str());
                }
            }
            res
        }
        .instrument(span)
        .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
            "validation"
        );
        next.run(ctx).instrument(span).await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let operation = operation_name.unwrap_or("anonymous");

        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
            "graphql",
            otel.name = format!("graphql {operation}"),
            operation = operation,
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );

        let response = next.run(ctx, operation_name).instrument(span.clone()).await;

        if let Some(error) = response.errors.first() {
            crate::tracing_context::record_error(&span, &error.message);
        }

        response
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        if info.is_for_introspection || info.path_node.parent.is_some() {
            return next
                .run(ctx, info)
                .inspect_err(|err| {
                    tracing::error!(
                        target: "async_graphql::graphql",
                        error = %err.message,
                        "error",
                    );
                })
                .await;
        }

        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
            "field",
            otel.name = format!("{}.{}", info.parent_type, info.name),
            path = %info.path_node,
            return_type = info.return_type,
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );

        next.run(ctx, info)
            .inspect_err(|err| {
                tracing::error!(
                    target: "async_graphql::graphql",
                    error = %err.message,
                    "error",
                );
                crate::tracing_context::record_current_error(&err.message);
            })
            .instrument(span)
            .await
    }
}
