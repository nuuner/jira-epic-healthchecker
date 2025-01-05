use crate::AppState;
use crate::time_graph::render_issue_time_graph;

pub async fn issue_svg(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(issue_key): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let issue_log = state
        .database
        .get_logs_of_issue(&issue_key)
        .await
        .expect("Could not get issue log");
    let svg_content = render_issue_time_graph(issue_log).await;
    
    (
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        svg_content,
    )
} 