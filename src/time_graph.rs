use crate::models::IssueLog;
use crate::renderer::{Renderer, Point, GraphStyle};
use axum;
use chrono;

pub struct TimeSpentData {
    date: i64,
    time_spent: i64,
    time_estimate: i64,
}

pub struct TimeGraph {
    renderer: Renderer,
}

impl TimeGraph {
    pub fn new(width: i32, height: i32, margin: i32) -> Self {
        Self {
            renderer: Renderer::new(width, height, margin),
        }
    }

    fn normalize_time_data(&self, time_spent_data: &[TimeSpentData]) -> (Vec<Point>, Vec<Point>) {
        if time_spent_data.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let min_date = time_spent_data.first().unwrap().date;
        let max_date = time_spent_data.last().unwrap().date;
        let max_value = time_spent_data
            .iter()
            .map(|data| data.time_spent.max(data.time_estimate))
            .max()
            .unwrap();

        let normalize_value = |value: i64| {
            if max_value == 0 {
                self.renderer.height as f64
            } else {
                self.renderer.height as f64 - (value as f64 / max_value as f64) * self.renderer.height as f64
            }
        };

        let normalize_x = |date: i64| {
            if max_date == min_date {
                0.0
            } else {
                (date - min_date) as f64 / (max_date - min_date) as f64 * self.renderer.width as f64
            }
        };

        let spent_points: Vec<Point> = time_spent_data
            .iter()
            .map(|data| Point {
                x: normalize_x(data.date),
                y: normalize_value(data.time_spent),
            })
            .collect();

        let estimate_points: Vec<Point> = time_spent_data
            .iter()
            .map(|data| Point {
                x: normalize_x(data.date),
                y: normalize_value(data.time_estimate),
            })
            .collect();

        (spent_points, estimate_points)
    }

    pub fn render(&mut self, time_spent_data: Vec<TimeSpentData>) -> String {
        let (spent_points, estimate_points) = self.normalize_time_data(&time_spent_data);
        
        let graphs = vec![
            (estimate_points, GraphStyle {
                stroke_color: "#4A90E2",
                fill_color: "#4A90E2",
            }),
            (spent_points, GraphStyle {
                stroke_color: "#E74C3C",
                fill_color: "#E74C3C", 
            }),
        ];
        
        self.renderer.render_line_graphs(graphs)
    }
}

pub async fn render_issue_time_graph(issue_logs: Vec<IssueLog>) -> axum::response::Html<String> {
    let mut time_graph = TimeGraph::new(150, 50, 10);

    let mut time_spent_data = issue_logs
        .into_iter()
        .map(|log| TimeSpentData {
            date: chrono::DateTime::parse_from_rfc3339(&log.updated_at)
                .unwrap()
                .timestamp(),
            time_spent: log.time_spent,
            time_estimate: log.time_estimate,
        })
        .collect::<Vec<_>>();

    time_spent_data.sort_by(|a, b| a.date.cmp(&b.date));
    
    axum::response::Html(time_graph.render(time_spent_data))
} 