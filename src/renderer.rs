use svg;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct GraphStyle {
    pub stroke_color: &'static str,
    pub fill_color: &'static str,
}

pub struct Renderer {
    document: svg::Document,
    pub width: i32,
    pub height: i32,
    margin: i32,
}

impl Renderer {
    pub fn new(width: i32, height: i32, margin: i32) -> Self {
        Self {
            document: svg::Document::new(),
            width,
            height,
            margin,
        }
    }

    pub fn render_line_graphs(&mut self, graphs: Vec<(Vec<Point>, GraphStyle)>) -> String {
        let mut document = svg::Document::new()
            .set(
                "viewBox",
                (-self.margin, -self.margin, self.width + 2 * self.margin, self.height + 2 * self.margin),
            )
            .set("width", self.width + 2 * self.margin)
            .set("height", self.height + 2 * self.margin);

        for (points, style) in graphs {
            if !points.is_empty() {
                document = document.add(self.get_filled_area(&points, style.fill_color));
                document = document.add(self.get_path(&points, style.stroke_color));
            }
        }

        self.document = document;
        self.document.to_string()
    }

    fn get_path(&self, points: &[Point], color: &str) -> svg::node::element::Path {
        let mut path_data = svg::node::element::path::Data::new();
        if let Some(first) = points.first() {
            path_data = path_data.move_to((first.x, first.y));
            for point in points {
                path_data = path_data.line_to((point.x, point.y));
            }
        }
        svg::node::element::Path::new()
            .set("d", path_data)
            .set("stroke", color)
            .set("stroke-width", "2")
            .set("fill", "none")
    }

    fn get_filled_area(&self, points: &[Point], color: &str) -> svg::node::element::Path {
        let mut path_data = svg::node::element::path::Data::new();
        if let Some(first) = points.first() {
            path_data = path_data.move_to((first.x, self.height as f64));
            path_data = path_data.line_to((first.x, first.y));
            for point in points {
                path_data = path_data.line_to((point.x, point.y));
            }
            if let Some(last) = points.last() {
                path_data = path_data.line_to((last.x, self.height as f64));
            }
            path_data = path_data.close();
        }
        svg::node::element::Path::new()
            .set("d", path_data)
            .set("fill", color)
            .set("fill-opacity", "0.5")
            .set("stroke", "none")
    }
}
