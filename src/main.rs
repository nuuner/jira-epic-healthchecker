mod collector;
mod database;
mod jira_client;
mod models;
mod renderer;
mod time_graph;

use collector::*;
use database::*;
use jira_client::*;
use time_graph::render_issue_time_graph;

#[derive(Clone)]
struct AppState {
    database: std::sync::Arc<Database>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tokio::spawn(async {
        run_data_collector().await;
    });

    let state = AppState {
        database: std::sync::Arc::new(
            Database::new()
                .await
                .expect("Could not create database for collector"),
        ),
    };

    let app = axum::Router::new()
        .route("/", axum::routing::get(root))
        .route("/issue/{issue_key}/time_graph.svg", axum::routing::get(issue_svg))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not bind to port 8080");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}

async fn issue_svg(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(issue_key): axum::extract::Path<String>,
) -> axum::response::Html<String> {
    let issue_log = state
        .database
        .get_logs_of_issue(&issue_key)
        .await
        .expect("Could not get issue log");
    render_issue_time_graph(issue_log).await
}

async fn root(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Html<String> {
    let epics = state
        .database
        .get_epics()
        .await
        .expect("Could not get epics");

    axum::response::Html(format!(
        r#"
        <html>
            <body>
                <h1>Epics</h1>
                <div>
                    {}
                </div>
            </body>
        </html>
        "#,
        epics
            .iter()
            .map(|e| format!("{}: {}", e.key, e.summary))
            .collect::<Vec<_>>()
            .join("<br>")
    ))
}

async fn run_data_collector() {
    let jira_client = JiraClient::new();
    let database = Database::new()
        .await
        .expect("Could not create database for collector");
    loop {
        if let Err(e) = collect_data(&jira_client, &database).await {
            println!("Error collecting data: {}", e);
        }

        println!("Sleeping for 10 minutes...");
        tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
    }
}
