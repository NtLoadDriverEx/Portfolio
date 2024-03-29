use eframe::egui;

use crate::easy_mark_viewer::easy_mark;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

// Define structs that match the TOML structure
#[derive(serde::Deserialize)]
struct TextContents {
    work_experience: WorkExperience,
    biography: TextBlock,
    about_page: TextBlock,
}

#[derive(serde::Deserialize)]
struct TextBlock {
    text: String,
}

#[derive(serde::Deserialize)]
struct WorkExperience {
    lucid_software: ExperienceDetails,
    freelance_projects: ExperienceDetails,
}

#[derive(serde::Deserialize)]
struct ExperienceDetails {
    description: String,
}

/// We derive Deserialize/Serialize, so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PortfolioApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    parsed_text: TextContents,

    #[serde(skip)]
    background: Background,

    about_page: bool,
}

impl Default for PortfolioApp {
    fn default() -> Self {
        let text_contents: TextContents =
            toml::from_str(include_str!("../assets/text_contents.toml"))
                .expect("Failed to parse TOML");
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            parsed_text: text_contents,
            background: Background::default(),
            about_page: true,
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
        let text_window_max_width: f32 = screen_size.width() - screen_size.width() * 0.2;

        if !self.background.has_points() {
            self.background.add_points(screen_size)
        }

        let input = ctx.input(|input| input.clone());
        self.background.update_points(
            ctx.pointer_latest_pos().unwrap_or(egui::Pos2::ZERO),
            screen_size,
            &input.unstable_dt,
        );

        self.background.calculate_collisions();

        let painter = ctx.layer_painter(egui::LayerId::background());
        self.background.render_draw_data(painter);

        let clear_frame = egui::Frame {
            fill: egui::Color32::from_rgba_premultiplied(0, 0, 0, 0),
            ..egui::Frame::default()
        };

        egui::TopBottomPanel::top("top_panel")
            .frame(clear_frame)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                    let source_link = egui::Hyperlink::from_label_and_url(
                        "Source",
                        "https://github.com/NtLoadDriverEx/Portfolio",
                    );
                    ui.add(source_link);

                    if ui.selectable_label(self.about_page, "About").clicked() {
                        self.about_page = !self.about_page;
                    }
                });
            });

        let about_window = egui::Window::new("About").auto_sized();
        if self.about_page {
            about_window.show(ctx, |ui| {
                ui.allocate_ui(egui::vec2(250f32.min(text_window_max_width), 150.), |ui| {
                    easy_mark(ui, &self.parsed_text.about_page.text);
                });
            });
        }

        egui::Window::new("Bio").auto_sized().show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(500f32.min(text_window_max_width), 2000.), |ui| {
                easy_mark(ui, &self.parsed_text.biography.text);
            });
        });

        egui::Window::new("Experience")
            .auto_sized()
            .show(ctx, |ui| {
                ui.allocate_ui(egui::vec2(700f32.min(text_window_max_width), 1000.), |ui| {
                    let lucid = &self.parsed_text.work_experience.lucid_software;
                    easy_mark(ui, &lucid.description);

                    let freelance = &self.parsed_text.work_experience.freelance_projects;
                    easy_mark(ui, &freelance.description);
                });
            });

        // draw in continuous mode.
        ctx.request_repaint();
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

const PT_LINE_DISTANCE: f32 = 120.0;

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
    xv: f32,
    yv: f32,
}

impl Point {
    // Normalize the point's velocity
    fn distance(p1: &Point, p2: &Point) -> f32 {
        ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
    }
    // Function to calculate the unit vector from one point to another
    fn direction_to(&self, other: &Point) -> Point {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dist = Point::distance(self, other);
        Point {
            x: dx / dist,
            y: dy / dist,
            xv: 0.0,
            yv: 0.0,
        }
    }

    // Elastic collision logic, assuming equal mass for simplicity
    fn elastic_collision(p1: &mut Point, p2: &mut Point) {
        let unit_normal = p1.direction_to(p2);
        let unit_tangent = Point {
            x: -unit_normal.y,
            y: unit_normal.x,
            xv: 0.0,
            yv: 0.0,
        };

        // Decompose velocities into normal and tangent components
        let v1n = unit_normal.x * p1.xv + unit_normal.y * p1.yv;
        let v1t = unit_tangent.x * p1.xv + unit_tangent.y * p1.yv;

        let v2n = unit_normal.x * p2.xv + unit_normal.y * p2.yv;
        let v2t = unit_tangent.x * p2.xv + unit_tangent.y * p2.yv;

        // For elastic collisions, the velocities along the normal direction exchange
        // since we assume equal mass, the formula simplifies
        let v1n_final = v2n;
        let v2n_final = v1n;

        // Update velocities, the tangent component remains unchanged
        p1.xv = v1n_final * unit_normal.x + v1t * unit_tangent.x;
        p1.yv = v1n_final * unit_normal.y + v1t * unit_tangent.y;

        p2.xv = v2n_final * unit_normal.x + v2t * unit_tangent.x;
        p2.yv = v2n_final * unit_normal.y + v2t * unit_tangent.y;
    }
}

// Implementing the Sub trait to overload the '-' operator for Point
impl<'a, 'b> std::ops::Sub<&'b Point> for &'a Point {
    type Output = Point;

    fn sub(self, other: &'b Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            xv: self.xv - other.xv,
            yv: self.yv - other.yv,
        }
    }
}

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

        let num_points: i32 = ((screen_size.width() / 3.0) * 0.5) as i32;

        for _ in 0..num_points {
            new_points.push(Point {
                x: rng.gen_range(0.0..screen_size.width()),
                y: rng.gen_range(0.0..screen_size.height()),
                xv: rng.gen_range(-20.0..=20.0),
                yv: rng.gen_range(-20.0..=20.0),
            });
        }
        self.points = new_points;
    }

    fn update_points(&mut self, mouse_pos: egui::Pos2, screen_size: egui::Rect, dt: &f32) {
        for point in &mut self.points {
            // Update point velocity and position

            point.x += point.xv * dt * 10.;
            point.y += point.yv * dt * 10.;

            //point.xv += 9.8 * dt * 10.;
            point.yv += 0.98 * dt * 10.;

            // Repel from mouse cursor
            let distance_to_mouse = (egui::Pos2::new(point.x, point.y) - mouse_pos).length();
            if distance_to_mouse < 60.0 {
                let direction = (egui::Pos2::new(point.x, point.y) - mouse_pos).normalized();
                point.xv += direction.x * 30.;
                point.yv += direction.y * 30.;
            }

            point.xv *= 0.999;
            point.yv *= 0.999;

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

            // Remove points from active intervals that are too far behind the current point
            active_intervals.retain(|&j| self.points[j].x + PT_LINE_DISTANCE >= point.x);

            // Check for collisions within the active intervals
            for &j in &active_intervals {
                // Ensure we only check collisions for distinct points
                if j != i {
                    let distance = (self.points[j].x - point.x).abs();
                    if distance <= PT_LINE_DISTANCE {
                        // Since we are dealing with 1D intervals, a simple distance check is sufficient
                        // For 2D or 3D, you'd need more complex overlap checks
                        self.collisions.push((j, i));
                    }
                }
            }

            // Add the current point to active intervals
            active_intervals.push(i);
        }
    }

    fn get_two_mut(slice: &mut [Point], idx1: usize, idx2: usize) -> (&mut Point, &mut Point) {
        assert!(idx1 != idx2, "Indices must be distinct");
        let (first, second) = if idx1 < idx2 {
            (idx1, idx2)
        } else {
            (idx2, idx1)
        };
        let (head, tail) = slice.split_at_mut(second);
        (&mut head[first], &mut tail[0])
    }

    // Returns data for egui to draw, not drawing directly
    fn prepare_draw_data(&mut self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();

        for &point in &self.points {
            commands.push(DrawCommand::Circle {
                center: egui::Pos2::new(point.x, point.y),
                radius: 5.0,
                color: egui::Color32::from_gray(200),
            });
        }

        for &(idx1, idx2) in &self.collisions {
            let (point, other) = Background::get_two_mut(&mut self.points, idx1, idx2);
            let dist = Point::distance(point, other);

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
            if dist < 5.0 {
                Point::elastic_collision(point, other);
            }
        }

        commands
    }

    fn render_draw_data(&mut self, painter: egui::Painter) {
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
