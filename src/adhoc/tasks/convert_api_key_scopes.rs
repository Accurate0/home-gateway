use sqlx::{Postgres, Transaction};

use crate::adhoc::{AdhocTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct ConvertApiKeyScopes;

#[async_trait::async_trait]
impl AdhocTask for ConvertApiKeyScopes {
    fn ordinal(&self) -> i64 {
        1
    }

    fn name(&self) -> &'static str {
        "convert_api_key_scopes"
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError> {
        Ok(convert(ctx.tx).await?)
    }
}

async fn convert(tx: &mut Transaction<'static, Postgres>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query!("SELECT id, name, scopes FROM api_keys")
        .fetch_all(&mut **tx)
        .await?;

    for row in rows {
        let converted = convert_all(&row.scopes);

        if converted == row.scopes {
            continue;
        }

        tracing::info!(
            key = %row.name,
            from = ?row.scopes,
            to = ?converted,
            "converting api key scopes to the new format"
        );

        sqlx::query!(
            "UPDATE api_keys SET scopes = $2 WHERE id = $1",
            row.id,
            &converted
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

fn convert_all(scopes: &[String]) -> Vec<String> {
    let mut converted = Vec::new();

    for scope in scopes {
        let mapped = convert_one(scope);

        if mapped.is_empty() {
            tracing::warn!("dropping scope with no equivalent in the new format: {scope}");
        }

        for scope in mapped {
            if !converted.contains(&scope) {
                converted.push(scope);
            }
        }
    }

    converted
}

fn convert_one(scope: &str) -> Vec<String> {
    if scope.trim() == "*" {
        return vec!["**:*".to_owned()];
    }

    let parts: Vec<&str> = scope.trim().split(':').collect();
    let [domain, resource, action] = parts.as_slice() else {
        return Vec::new();
    };

    let action = match *action {
        "read" => "read",
        "write" | "execute" => "write",
        "*" => "*",
        _ => return Vec::new(),
    };

    let prefix = match *domain {
        "graphql" | "rest" => "",
        "admin" => "admin.",
        "ingest" => "ingest.",
        "events" => "events.",
        "*" => return Vec::new(),
        _ => return Vec::new(),
    };

    if *resource == "*" {
        return match prefix {
            "" => vec![format!("*:{action}"), format!("media.player:{action}")],
            _ => vec![format!("{prefix}*:{action}")],
        };
    }

    let resource = match (prefix, *resource) {
        ("", "media_player") => "media.player".to_owned(),
        ("", "roborock" | "valetudo") => "robot_vacuum".to_owned(),
        ("", "entity") => return Vec::new(),
        (_, resource) => format!("{prefix}{resource}"),
    };

    vec![format!("{resource}:{action}")]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn global_wildcard_becomes_the_new_global() {
        assert_eq!(convert_one("*"), ["**:*"]);
    }

    #[test]
    fn the_domain_segment_is_dropped() {
        assert_eq!(convert_one("graphql:solar:read"), ["solar:read"]);
        assert_eq!(convert_one("rest:epd:read"), ["epd:read"]);
    }

    #[test]
    fn admin_ingest_and_events_become_path_prefixes() {
        assert_eq!(convert_one("admin:keys:write"), ["admin.keys:write"]);
        assert_eq!(convert_one("ingest:unifi:write"), ["ingest.unifi:write"]);
        assert_eq!(
            convert_one("events:presence:read"),
            ["events.presence:read"]
        );
    }

    #[test]
    fn execute_folds_into_write() {
        assert_eq!(convert_one("rest:workflow:execute"), ["workflow:write"]);
        assert_eq!(
            convert_one("graphql:adhoc_task:execute"),
            ["adhoc_task:write"]
        );
    }

    #[test]
    fn renamed_resources_are_carried_over() {
        assert_eq!(
            convert_one("graphql:media_player:write"),
            ["media.player:write"]
        );
        assert_eq!(convert_one("graphql:roborock:read"), ["robot_vacuum:read"]);
    }

    #[test]
    fn a_resource_wildcard_expands_to_reach_nested_resources() {
        assert_eq!(
            convert_one("graphql:*:read"),
            ["*:read", "media.player:read"]
        );
        assert_eq!(convert_one("ingest:*:write"), ["ingest.*:write"]);
    }

    #[test]
    fn unmappable_scopes_are_dropped() {
        assert!(convert_one("graphql:entity:read").is_empty());
        assert!(convert_one("bogus:solar:read").is_empty());
        assert!(convert_one("graphql:solar").is_empty());
    }

    #[test]
    fn conversion_dedupes_across_the_whole_list() {
        let scopes = [
            "graphql:epd:read".to_owned(),
            "rest:epd:read".to_owned(),
            "graphql:entity:read".to_owned(),
        ];

        assert_eq!(convert_all(&scopes), ["epd:read"]);
    }

    #[test]
    fn a_whole_key_converts_as_a_unit() {
        let scopes = [
            "graphql:*:read".to_owned(),
            "admin:keys:write".to_owned(),
            "ingest:home:write".to_owned(),
            "events:media_player:read".to_owned(),
            "graphql:adhoc_task:execute".to_owned(),
        ];

        assert_eq!(
            convert_all(&scopes),
            [
                "*:read",
                "media.player:read",
                "admin.keys:write",
                "ingest.home:write",
                "events.media_player:read",
                "adhoc_task:write",
            ]
        );
    }
}
