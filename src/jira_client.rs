use crate::models::*;
use std::env;

pub struct JiraClient {
    client: reqwest::Client,
    base_url: String,
}

impl JiraClient {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", env::var("JIRA_TOKEN").expect("JIRA_TOKEN must be set"))
                .parse()
                .unwrap(),
        );

        let base_url = env::var("JIRA_BASE_URL").expect("JIRA_BASE_URL must be set");

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url,
        }
    }

    async fn _get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<T>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_myself(&self) -> Result<JiraUser, String> {
        self._get("/rest/api/2/myself").await
    }

    pub async fn get_jql(&self, jql: &str) -> Result<Vec<JiraIssue>, String> {
        let mut issues: Vec<JiraIssue> = Vec::new();
        let mut start_at = 0;

        loop {
            let issues_response = self
                ._get::<IssueListResponse>(&format!(
                    "/rest/api/2/search?jql={}&startAt={}",
                    jql, start_at
                ))
                .await?;

            let total = issues_response.total;
            let page_size = issues_response.issues.len();

            issues.extend(issues_response.issues);
            start_at += page_size;

            if start_at >= total.try_into().unwrap() {
                break;
            }
        }

        Ok(issues)
    }
}
