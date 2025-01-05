use crate::models::*;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create storage directory: {0}")]
    StorageCreation(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("Failed to parse count: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

impl From<tokio_rusqlite::Error> for DatabaseError {
    fn from(err: tokio_rusqlite::Error) -> Self {
        DatabaseError::Sqlite(err.to_string())
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::Sqlite(err.to_string())
    }
}

pub struct Database {
    connection: tokio_rusqlite::Connection,
}

impl Database {
    pub async fn new() -> Result<Self, DatabaseError> {
        if !std::path::Path::new("storage/").exists() {
            fs::create_dir("storage")?;
        }
        let connection = tokio_rusqlite::Connection::open("storage/jira_health_checker.db").await?;
        let db = Database { connection };
        db._init_database().await?;
        Ok(db)
    }

    async fn _init_database(&self) -> Result<(), DatabaseError> {
        let check_if_tables_exist = "
            SELECT COUNT(*) 
            FROM sqlite_schema 
            WHERE type = 'table' AND name IN ('epics', 'issues')";

        let result = self
            .connection
            .call(|conn| {
                let mut statement = conn.prepare(check_if_tables_exist)?;
                statement
                    .query_row([], |row| row.get::<_, i64>(0))
                    .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;

        if result == 0 {
            let init = "
                CREATE TABLE IF NOT EXISTS epics (
                    key TEXT NOT NULL PRIMARY KEY,
                    summary TEXT NOT NULL,
                    updated_at DATETIME NOT NULL
                );
                CREATE TABLE IF NOT EXISTS issues (
                    key TEXT NOT NULL,
                    summary TEXT NOT NULL,
                    epic_key TEXT NOT NULL,
                    time_estimate INTEGER NOT NULL,
                    time_spent INTEGER NOT NULL,
                    updated_at DATETIME NOT NULL,
                    assignee TEXT NOT NULL,
                    FOREIGN KEY (epic_key) REFERENCES epics(key)
                )";
            self.connection
                .call(|conn| {
                    conn.execute(init, [])
                        .map_err(tokio_rusqlite::Error::Rusqlite)
                })
                .await?;
        }
        Ok(())
    }

    pub async fn insert_epic(&self, epic: &JiraIssue) -> Result<(), DatabaseError> {
        let query = "
            INSERT INTO epics (key, summary, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                summary = excluded.summary,
                updated_at = excluded.updated_at
        ";
        let key = epic.key.clone();
        let summary = epic.fields.summary.clone();
        let timestamp = chrono::Utc::now().to_rfc3339();

        self.connection
            .call(move |conn| {
                conn.execute(query, (key.as_str(), summary.as_str(), timestamp.as_str()))
                    .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;
        Ok(())
    }

    pub async fn insert_issue(&self, issue: &JiraIssue) -> Result<(), DatabaseError> {
        let query = "
            INSERT INTO issues (key, summary, epic_key, time_estimate, time_spent, updated_at, assignee)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ";
        let key = issue.key.clone();
        let summary = issue.fields.summary.clone();
        let epic_key = issue
            .fields
            .epic_key
            .as_ref()
            .expect("Epic key is missing")
            .clone();
        let time_estimate = issue.fields.time_estimate.unwrap_or(0);
        let time_spent = issue.fields.time_spent.unwrap_or(0);
        let timestamp = chrono::Utc::now().to_rfc3339();
        let assignee = issue
            .fields
            .assignee
            .as_ref()
            .map(|assignee| assignee.name.clone())
            .unwrap_or_default();

        self.connection
            .call(move |conn| {
                conn.execute(
                    query,
                    (
                        key.as_str(),
                        summary.as_str(),
                        epic_key.as_str(),
                        time_estimate as i64,
                        time_spent as i64,
                        timestamp.as_str(),
                        assignee.as_str(),
                    ),
                )
                .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;
        Ok(())
    }

    pub async fn get_logs_of_issue(&self, issue_key: &str) -> Result<Vec<IssueLog>, DatabaseError> {
        let query = "
            SELECT * FROM issues WHERE key = ?
        ";
        let issue_key = issue_key.to_string();
        let logs = self
            .connection
            .call(move |conn| {
                let mut stmt = conn
                    .prepare_cached(query)
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                let rows = stmt.query_map([issue_key.as_str()], |row| {
                    Ok(IssueLog {
                        key: row.get("key")?,
                        summary: row.get("summary")?,
                        epic_key: row.get("epic_key")?,
                        time_estimate: row.get("time_estimate")?,
                        time_spent: row.get("time_spent")?,
                        updated_at: row.get("updated_at")?,
                        assignee: row.get("assignee")?,
                    })
                })
                .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))
            })
            .await?;
        Ok(logs)
    }

    pub async fn get_all_latest_issue_logs(&self) -> Result<Vec<IssueLog>, DatabaseError> {
        let query = "
            SELECT *
            FROM (
                SELECT *,
                    ROW_NUMBER() OVER (PARTITION BY key ORDER BY updated_at DESC) as rn
                FROM issues
            ) ranked
            WHERE rn = 1
        ";
        
        let logs = self
            .connection
            .call(|conn| {
                let mut stmt = conn
                    .prepare_cached(query)
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                let rows = stmt
                    .query_map([], |row| {
                        Ok(IssueLog {
                            key: row.get("key")?,
                            summary: row.get("summary")?,
                            epic_key: row.get("epic_key")?,
                            time_estimate: row.get("time_estimate")?,
                            time_spent: row.get("time_spent")?,
                            updated_at: row.get("updated_at")?,
                            assignee: row.get("assignee")?,
                        })
                    })
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))
            })
            .await?;

        Ok(logs)
    }

    pub async fn get_epics(&self) -> Result<Vec<Epic>, DatabaseError> {
        const QUERY: &str = r#"
            SELECT 
                key,
                summary 
            FROM epics
            ORDER BY key
        "#;

        let epics = self
            .connection
            .call(|conn| {
                let mut stmt = conn
                    .prepare_cached(QUERY)
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                let rows = stmt
                    .query_map([], |row| {
                        Ok(Epic {
                            key: row.get("key")?,
                            summary: row.get("summary")?,
                        })
                    })
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;

                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))
            })
            .await?;

        Ok(epics)
    }
}
