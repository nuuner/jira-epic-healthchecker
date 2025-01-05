use crate::database::*;
use crate::jira_client::*;
use std::env;

pub async fn collect_data(jira_client: &JiraClient, database: &Database) -> Result<(), String> {
    println!("Collecting data...");

    let epics_jql = env::var("JIRA_EPICS_JQL").expect("JIRA_EPICS_JQL must be set");
    let issues_jql = env::var("JIRA_ISSUES_JQL").expect("JIRA_ISSUES_JQL must be set");

    let epics = jira_client
        .get_jql(&format!("{} AND type = Epic", epics_jql))
        .await
        .expect("Could not get epics");
    let epics_csv = epics
        .iter()
        .map(|epic| format!("{}", epic.key))
        .collect::<Vec<String>>()
        .join(",");

    for epic in epics {
        println!("Epic {}: {}", epic.key, epic.fields.summary);
        database
            .insert_epic(&epic)
            .await
            .expect("Could not insert epic");
    }

    let issues = jira_client
        .get_jql(&format!("{} AND 'Epic Link' IN ({})", issues_jql, epics_csv))
        .await
        .expect("Could not get issues");
    for issue in issues {
        println!(
            "Collecting issue {}: {}, {}, {}, {}",
            issue.key,
            issue.fields.summary,
            issue.fields.time_estimate.unwrap_or(0),
            issue.fields.time_spent.unwrap_or(0),
            issue
                .fields
                .assignee
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or("unassigned".to_string())
        );
        database
            .insert_issue(&issue)
            .await
            .expect("Could not insert issue");
    }

    println!("Data collected and inserted into database");
    Ok(())
}
