use crate::AppState;
use itertools::Itertools;

pub async fn root(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Html<String> {
    let epics = state
        .database
        .get_epics()
        .await
        .expect("Could not get epics");

    let issue_logs = state
        .database
        .get_all_latest_issue_logs()
        .await
        .expect("Could not get issue logs");

    let issues_by_epic = issue_logs
        .iter()
        .into_group_map_by(|issue_log| issue_log.epic_key.clone());

    let issues_by_epic_html = issues_by_epic
        .iter()
        .map(|(epic_key, issues)| format!(
            "<h2>{}</h2><ul>{}</ul>",
            epics.iter().find(|e| e.key == *epic_key).unwrap().summary,
            issues.iter().map(|issue| format!("
            <li>
                <div>
                    <span>{}: {}</span><br>
                    <button hx-get=\"/issue/{}/time_graph.svg\" hx-target=\"this\" hx-swap=\"outerHTML\">View Time Graph</button>
                </div>
            </li>", issue.key, issue.summary, issue.key)).collect::<Vec<_>>().join("")
        ))
        .collect::<Vec<_>>().join("");

    axum::response::Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script src="/static/htmx.min.js"></script>
            </head>
            <body>
                <h1>Epics</h1>
                <div>
                    {}
                </div>
            </body>
        </html>
        "#,
        issues_by_epic_html
    ))
} 