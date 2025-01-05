mod collector;
mod database;
mod jira_client;
mod models;
mod renderer;
mod routes;
mod time_graph;

use collector::*;
use database::*;
use jira_client::*;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
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
        .route("/", axum::routing::get(routes::root))
        .route("/issue/{issue_key}/time_graph.svg", axum::routing::get(routes::issue_svg))
        .nest_service("/static", ServeDir::new("src/static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not bind to port 8080");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
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
