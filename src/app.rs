use eframe::egui;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
    xv: f32,
    yv: f32,
}

/// We derive Deserialize/Serialize, so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PortfolioApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    background: Background,

    #[serde(skip)] // This how you opt out of serialization of a field
    value: f32,
}

impl Default for PortfolioApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            background: Background::default(),
        }
    }
}

impl PortfolioApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for PortfolioApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screen_size = ctx.screen_rect();
        if !self.background.has_points() {
            let mut new_points: Vec<Point> = Vec::new();

            let mut rng = ChaCha20Rng::from_entropy();

            let num_points: i32 = ((screen_size.width() / 3.0) * 0.2) as i32;

            for _ in 0..num_points {
                new_points.push(Point {
                    x: rng.gen_range(0.0..screen_size.width()),
                    y: rng.gen_range(0.0..screen_size.height()),
                    xv: rng.gen_range(-12.0..=12.0),
                    yv: rng.gen_range(-12.0..=12.0),
                });
            }

            self.background.add_points(new_points);
        }

        self.background.update_points(
            ctx.pointer_latest_pos().unwrap_or(egui::Pos2::ZERO),
            egui::vec2(screen_size.width(), screen_size.height()),
        );
        self.background.calculate_collisions();

        let commands = self.background.prepare_draw_data();
        let painter = ctx.layer_painter(egui::LayerId::background());

        for command in commands {
            match command {
                DrawCommand::Circle {
                    center,
                    radius,
                    color,
                } => {
                    painter.circle_filled(center, radius, color);
                }
                DrawCommand::Line {
                    points,
                    width,
                    color,
                } => {
                    painter.line_segment(points, (width, color));
                }
            }
        }
        // draw in continuous mode.
        ctx.request_repaint();
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn distance(p1: egui::Pos2, p2: egui::Pos2) -> f32 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}
const PT_LINE_DISTANCE: f32 = 120.0;
struct Background {
    points: Vec<Point>,
    collisions: Vec<(usize, usize)>, // Stores tuples of indices of colliding points
}

impl Background {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            collisions: Vec::new(),
        }
    }

    fn has_points(&self) -> bool {
        !self.points.is_empty()
    }

    fn add_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }

    fn update_points(&mut self, mouse_pos: egui::Pos2, screen_size: egui::Vec2) {
        for point in &mut self.points {
            // Update point velocity and position
            point.x += 0.1 * point.xv;
            point.y += 0.1 * point.yv;

            // Repel from mouse cursor
            let distance_to_mouse = (egui::Pos2::new(point.x, point.y) - mouse_pos).length();
            if distance_to_mouse < 60.0 {
                let direction = (egui::Pos2::new(point.x, point.y) - mouse_pos).normalized();
                point.xv += direction.x * 0.5;
                point.yv += direction.y * 0.5;
            }

            // Dampen velocity if above threshold
            point.xv *= if point.xv.abs() > 12.0 { 0.92 } else { 1.0 };
            point.yv *= if point.yv.abs() > 12.0 { 0.92 } else { 1.0 };

            // Wrap around screen edges
            point.x = point.x.rem_euclid(screen_size.x);
            point.y = point.y.rem_euclid(screen_size.y);
        }
    }

    // Sweep and Prune algorithm implemented based on (https://youtu.be/eED4bSkYCB8?si=ljSOo6s2a9xyHzkI&t=941)
    fn calculate_collisions(&mut self) {
        self.collisions.clear();
        let mut active_intervals: Vec<usize> = Vec::new();

        // Sort points by their x-coordinate for efficient collision detection
        let mut sorted_points: Vec<usize> = (0..self.points.len()).collect();
        sorted_points.sort_by_key(|&i| self.points[i].x as i32);

        for &i in &sorted_points {
            let point = &self.points[i];
            active_intervals.retain(|&j| (self.points[j].x + PT_LINE_DISTANCE) > point.x);

            for &j in &active_intervals {
                let range_inclusive =
                    (self.points[j].x - PT_LINE_DISTANCE)..=(self.points[j].x + PT_LINE_DISTANCE);
                if range_inclusive.contains(&point.x) {
                    self.collisions.push((j, i));
                }
            }

            active_intervals.push(i);
        }
    }

    // Returns data for egui to draw, not drawing directly
    fn prepare_draw_data(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();

        for &point in &self.points {
            commands.push(DrawCommand::Circle {
                center: egui::Pos2::new(point.x, point.y),
                radius: 3.0,
                color: egui::Color32::from_gray(200),
            });
        }

        for &(idx1, idx2) in &self.collisions {
            let point = &self.points[idx1];
            let other = &self.points[idx2];
            let dist = distance(
                egui::Pos2::new(point.x, point.y),
                egui::Pos2::new(other.x, other.y),
            );

            if dist < PT_LINE_DISTANCE {
                let opacity = (PT_LINE_DISTANCE - dist) / PT_LINE_DISTANCE;
                let color =
                    egui::Color32::from_rgba_unmultiplied(164, 171, 176, (opacity * 127.5) as u8);
                commands.push(DrawCommand::Line {
                    points: [
                        egui::Pos2::new(point.x, point.y),
                        egui::Pos2::new(other.x, other.y),
                    ],
                    width: 0.5,
                    color,
                });
            }
        }

        commands
    }
}

enum DrawCommand {
    Circle {
        center: egui::Pos2,
        radius: f32,
        color: egui::Color32,
    },
    Line {
        points: [egui::Pos2; 2],
        width: f32,
        color: egui::Color32,
    },
}
