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

    text: String,

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
            text: String::new(),
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
            self.background.add_points(screen_size)
        }

        self.background.update_points(
            ctx.pointer_latest_pos().unwrap_or(egui::Pos2::ZERO),
            screen_size,
        );
        self.background.calculate_collisions();

        let painter = ctx.layer_painter(egui::LayerId::background());
        self.background.render_draw_data(painter);

        // let _clear_frame = egui::Frame {
        //     fill: egui::Color32::from_rgba_premultiplied(0, 0, 0, 0),
        //     ..egui::Frame::default()
        // };

        egui::Window::new("Bio")
            .resizable(true)
            .default_width(500.)
            .default_height(500.)
            .show(ctx, |ui| {
                ui.heading("Stuart Downing");
                ui.label("Passionate problem-solver and self-taught developer. I am quick to grasp new ideas, and adept at programming, particularly in low-level languages such as C++ and Rust.\
                 I enjoy solving complicated problems, where I can develop my skills with precision and efficiency.");

        });

        egui::Window::new("Experience")
            .resizable(true)
            .max_size(egui::vec2(1000.0, 1000.0))
            .show(ctx, |ui| {
                ui.heading("Lucid Software, 2022 - 2024");
                ui.label("Role: Software Engineer");
                ui.text_edit_multiline(
                    &mut "Sole developer of android app for an educational platform.\nNotable features include document scanning using OpenCV.\nLearned Kotlin and applied in 1 month.\nWorked in a fast paced team environment.",
                );


                ui.heading("Freelance Projects, 2020 - 2023");
                ui.label("Role: Software Engineer/Project Manager");
                ui.text_edit_multiline(
                    &mut "Managed and engineered multiple projects in C/C++ involving Win32.
Used python to write high performance scripts for personal projects and commercial applications.
Used x86 assembly with C to write complicated hooking libraries and manipulate virtual memory.
Experience in writing assembly and usage of intrinsics.
Usage of SQL and databases and interoperability between Java and C++.
Experience in graphics libraries like DirectX and Vulkan.
Lots of experience debugging and reverse engineering application errors.
");
            });

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

    fn add_points(&mut self, screen_size: egui::Rect) {
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
        self.points = new_points;
    }

    fn update_points(&mut self, mouse_pos: egui::Pos2, screen_size: egui::Rect) {
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
            point.x = point.x.rem_euclid(screen_size.width());
            point.y = point.y.rem_euclid(screen_size.height());
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

    fn render_draw_data(&self, painter: egui::Painter) {
        for command in self.prepare_draw_data() {
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
