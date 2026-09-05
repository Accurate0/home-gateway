use sqlx::{Executor, Pool, Postgres};
use std::time::Duration;
use testcontainers::{
    ContainerAsync, GenericBuildableImage, GenericImage, ImageExt, ReuseDirective,
    core::{IntoContainerPort, WaitFor},
    runners::{AsyncBuilder, AsyncRunner},
};
use tokio::sync::OnceCell;
use uuid::Uuid;

const POSTGRES_IMAGE: &str = "home-gateway-postgres";
const POSTGRES_TAG: &str = "test";
const DOCKERFILE: &str = "Dockerfile.postgres";
const CONTAINER_NAME: &str = "home-gateway-test-postgres";
const TEMPLATE_DATABASE: &str = "home_gateway_test_template";
const CONNECT_ATTEMPTS: u32 = 60;

static SERVER: OnceCell<Server> = OnceCell::const_new();

struct Server {
    admin_url: String,
    _container: Option<ContainerAsync<GenericImage>>,
}

pub struct TestDb {
    pub pool: Pool<Postgres>,
}

fn with_database(url: &str, database: &str) -> String {
    let mut parsed: url::Url = url.parse().expect("DATABASE_URL is not a valid url");
    parsed.set_path(database);
    parsed.into()
}

async fn connect_with_retry(url: &str) -> Pool<Postgres> {
    let mut last_error = None;

    for _ in 0..CONNECT_ATTEMPTS {
        match home_gateway::db::connect(url).await {
            Ok(pool) => match pool.acquire().await {
                Ok(_) => return pool,
                Err(e) => last_error = Some(e.to_string()),
            },
            Err(e) => last_error = Some(e.to_string()),
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    panic!("could not connect to postgres at {url}: {last_error:?}");
}

async fn start_server() -> Server {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Server {
            admin_url: url,
            _container: None,
        };
    }

    // Migrations need TimescaleDB *and* pgmq, so the stock postgres image will
    // not do — build the repo's own one. Docker layer caching makes every run
    // after the first cheap, and `Dockerfile.postgres` has no `COPY`, so the
    // build context is just the dockerfile.
    let image = GenericBuildableImage::new(POSTGRES_IMAGE, POSTGRES_TAG)
        .with_dockerfile(DOCKERFILE)
        .build_image()
        .await
        .unwrap_or_else(|e| {
            panic!(
                "failed to build {POSTGRES_IMAGE}:{POSTGRES_TAG} from {DOCKERFILE}: {e}\n\
                 is the docker daemon reachable? alternatively point DATABASE_URL \
                 at a running TimescaleDB+pgmq instance to skip the build."
            )
        });

    let container = image
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_container_name(CONTAINER_NAME)
        .with_reuse(ReuseDirective::Always)
        .with_env_var("POSTGRES_PASSWORD", "test")
        .with_env_var("POSTGRES_DB", "home")
        .with_cmd(["postgres", "-c", "shared_preload_libraries=timescaledb"])
        .start()
        .await
        .unwrap_or_else(|e| panic!("failed to start the {POSTGRES_IMAGE} container: {e}"));

    let port = container
        .get_host_port_ipv4(5432.tcp())
        .await
        .expect("failed to resolve the mapped postgres port");

    Server {
        admin_url: format!("postgres://postgres:test@127.0.0.1:{port}/home"),
        _container: Some(container),
    }
}

async fn server() -> &'static Server {
    SERVER
        .get_or_init(|| async {
            let server = start_server().await;

            let admin = connect_with_retry(&server.admin_url).await;

            let stale: Vec<String> =
                sqlx::query_scalar("SELECT datname FROM pg_database WHERE datname LIKE 'test\\_%'")
                    .fetch_all(&admin)
                    .await
                    .expect("failed to list the stale test databases");

            for database in stale {
                admin
                    .execute(format!(r#"DROP DATABASE IF EXISTS "{database}" WITH (FORCE)"#).as_str())
                    .await
                    .expect("failed to drop a stale test database");
            }

            admin
                .execute(format!(r#"DROP DATABASE IF EXISTS "{TEMPLATE_DATABASE}" WITH (FORCE)"#).as_str())
                .await
                .expect("failed to drop the stale template database");
            admin
                .execute(format!(r#"CREATE DATABASE "{TEMPLATE_DATABASE}""#).as_str())
                .await
                .expect("failed to create the template database");
            admin.close().await;

            let template_url = with_database(&server.admin_url, TEMPLATE_DATABASE);
            let template = connect_with_retry(&template_url).await;
            home_gateway::db::migrate(&template)
                .await
                .expect("failed to migrate the template database");

            // CREATE DATABASE ... TEMPLATE refuses to run while anything is
            // still connected to the template.
            template.close().await;

            server
        })
        .await
}

pub async fn fresh_database() -> TestDb {
    let server = server().await;
    let name = format!("test_{}", Uuid::new_v4().simple());

    let admin = connect_with_retry(&server.admin_url).await;
    admin
        .execute(format!(r#"CREATE DATABASE "{name}" TEMPLATE "{TEMPLATE_DATABASE}""#).as_str())
        .await
        .expect("failed to create the test database from the template");
    admin.close().await;

    let pool = connect_with_retry(&with_database(&server.admin_url, &name)).await;

    TestDb { pool }
}
