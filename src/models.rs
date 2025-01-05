use serde::Deserialize;

#[derive(Deserialize)]
pub struct JiraUser {
    pub name: String,
    #[serde(rename = "emailAddress")]
    pub email: String,
}

#[derive(Deserialize)]
pub struct IssueListResponse {
    pub expand: String,
    #[serde(rename = "startAt")]
    pub start_at: u64,
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    pub total: u64,
    pub issues: Vec<JiraIssue>,
}

#[derive(Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraIssueFields,
}

#[derive(Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    #[serde(rename = "customfield_11100")]
    pub epic_key: Option<String>,
    #[serde(rename = "aggregatetimeoriginalestimate")]
    pub time_estimate: Option<u64>,
    #[serde(rename = "aggregatetimespent")]
    pub time_spent: Option<u64>,
    pub assignee: Option<JiraUser>,
}

#[derive(Deserialize)]
pub struct Epic {
    pub key: String,
    pub summary: String,
}

#[derive(Deserialize)]
pub struct IssueLog {
    pub key: String,
    pub summary: String,
    pub epic_key: String,
    pub time_estimate: i64,
    pub time_spent: i64,
    pub updated_at: String,
    pub assignee: String,
}
