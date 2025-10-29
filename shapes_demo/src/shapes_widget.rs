use super::app::shapes_type::ShapeType;
use dust_dds::infrastructure::sample_info::InstanceStateKind;
use eframe::egui::{self, epaint};

const PURPLE: egui::Color32 = egui::Color32::from_rgb(128, 0, 128);
const BLUE: egui::Color32 = egui::Color32::BLUE;
const RED: egui::Color32 = egui::Color32::RED;
const GREEN: egui::Color32 = egui::Color32::GREEN;
const YELLOW: egui::Color32 = egui::Color32::YELLOW;
const CYAN: egui::Color32 = egui::Color32::from_rgb(0, 255, 255);
const MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);

#[derive(Clone)]
pub struct GuiShape {
    kind: String,
    color: egui::Color32,
    position: egui::Pos2,
    size: f32,
    instance_state_kind: InstanceStateKind,
}

impl GuiShape {
    pub fn from_shape_type(
        kind: String,
        shape_type: ShapeType,
        alpha: u8,
        instance_state_kind: InstanceStateKind,
    ) -> Self {
        let color = match shape_type.color.as_str() {
            "PURPLE" => PURPLE,
            "BLUE" => BLUE,
            "RED" => RED,
            "GREEN" => GREEN,
            "YELLOW" => YELLOW,
            "CYAN" => CYAN,
            "MAGENTA" => MAGENTA,
            "ORANGE" => ORANGE,
            _ => panic!("color not supported"),
        };
        let color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        Self {
            kind,
            color,
            position: egui::pos2(shape_type.x as f32, shape_type.y as f32),
            size: shape_type.shapesize as f32,
            instance_state_kind,
        }
    }

    pub fn as_shape_type(&self) -> ShapeType {
        let color = match egui::Color32::from_rgb(self.color.r(), self.color.g(), self.color.b()) {
            PURPLE => "PURPLE",
            BLUE => "BLUE",
            RED => "RED",
            GREEN => "GREEN",
            YELLOW => "YELLOW",
            CYAN => "CYAN",
            MAGENTA => "MAGENTA",
            ORANGE => "ORANGE",
            _ => panic!("Color not valid"),
        }
        .to_string();
        ShapeType {
            color,
            x: self.position.x as i32,
            y: self.position.y as i32,
            shapesize: self.size as i32,
        }
    }

    pub fn as_egui_shape(&self, scale: f32) -> egui::Shape {
        let stroke = egui::Stroke {
            width: 0.5,
            color: egui::Color32::BLACK,
        };

        let position = self.position * scale;
        let size = self.size * scale;
        let mut shape = Vec::<egui::Shape>::new();
        shape.push(match self.kind.as_str() {
            "Circle" => epaint::CircleShape {
                center: position,
                radius: size / 2.0,
                fill: self.color,
                stroke,
            }
            .into(),
            "Triangle" => epaint::PathShape {
                points: vec![
                    position + egui::vec2(0.0, -size / 2.0),
                    position + egui::vec2(-size / 2.0, size / 2.0),
                    position + egui::vec2(size / 2.0, size / 2.0),
                ],
                closed: true,
                fill: self.color,
                stroke: stroke.into(),
            }
            .into(),
            "Square" => epaint::RectShape::new(
                egui::Rect::from_center_size(position, epaint::vec2(size, size)),
                egui::Rounding::ZERO,
                self.color,
                stroke,
            )
            .into(),
            _ => panic!("shape kind not valid"),
        });
        let symbol_stroke = egui::Stroke::new(size / 20.0, stroke.color);

        let mut shape_position = position;
        let mut shape_size = size;
        if self.kind.as_str() == "Triangle" {
            shape_position.y += size * 0.2;
            shape_size *= 0.6;
        };
        let marker = match self.instance_state_kind {
            InstanceStateKind::NotAliveNoWriters => Some(question_mark_shape(
                egui::vec2(shape_size / 3.0, shape_size / 2.0),
                symbol_stroke,
            )),
            InstanceStateKind::NotAliveDisposed => {
                Some(cross_shape(shape_size * 0.6, symbol_stroke))
            }
            InstanceStateKind::Alive => None,
        };
        if let Some(mut marker) = marker {
            marker.translate(shape_position.to_vec2());
            shape.push(marker);
        }

        shape.into()
    }
}

#[derive(Clone)]
pub struct MovingShapeObject {
    gui_shape: GuiShape,
    velocity: egui::Vec2,
}

impl MovingShapeObject {
    pub fn new(shape: GuiShape, velocity: egui::Vec2) -> Self {
        Self {
            gui_shape: shape,
            velocity,
        }
    }

    pub fn move_within_rect(&mut self, rect_size: egui::Vec2, time_delta: f32) {
        let radius = self.gui_shape.size / 2.0;
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, rect_size);
        // Inset rect to account for shape size
        let rect = rect.expand(-radius);

        let reflection_normal = if self.gui_shape.position.x < rect.left() {
            self.gui_shape.position = egui::pos2(rect.left(), self.gui_shape.position.y);
            Some(egui::vec2(1.0, 0.0))
        } else if self.gui_shape.position.x > rect.right() {
            self.gui_shape.position = egui::pos2(rect.right(), self.gui_shape.position.y);
            Some(egui::vec2(-1.0, 0.0))
        } else if self.gui_shape.position.y < rect.top() {
            self.gui_shape.position = egui::pos2(self.gui_shape.position.x, rect.top());
            Some(egui::vec2(0.0, 1.0))
        } else if self.gui_shape.position.y > rect.bottom() {
            self.gui_shape.position = egui::pos2(self.gui_shape.position.x, rect.bottom());
            Some(egui::vec2(0.0, -1.0))
        } else {
            None
        };
        if let Some(normal) = reflection_normal {
            // reflect motion in respect to normal of surface
            self.velocity = self.velocity - 2.0 * (self.velocity * normal) * normal;
        }
        self.gui_shape.position += self.velocity * time_delta;
    }

    pub fn gui_shape(&self) -> &GuiShape {
        &self.gui_shape
    }
}

pub struct ShapesWidget<'a> {
    original_size: egui::Vec2,
    shape_list: &'a [GuiShape],
}

impl<'a> ShapesWidget<'a> {
    pub fn new(original_size: egui::Vec2, shape_list: &'a [GuiShape]) -> Self {
        Self {
            original_size,
            shape_list,
        }
    }

    fn paint_area_and_shapes(&self, ui: &mut egui::Ui) -> egui::Response {
        let max_size = ui.max_rect().size();
        let scale = if self.original_size.y / self.original_size.x > max_size.y / max_size.x {
            max_size.y / self.original_size.y
        } else {
            max_size.x / self.original_size.x
        };
        let desired_size = self.original_size * scale;
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
        painter.rect_filled(response.rect, egui::Rounding::ZERO, egui::Color32::WHITE);
        egui::Image::new(egui::include_image!("../res/s2e_logo_background.png"))
            .paint_at(ui, response.rect);
        for shape in self.shape_list {
            let mut shape = shape.as_egui_shape(scale);
            shape.translate(response.rect.left_top().to_vec2());
            painter.add(shape);
        }
        response
    }
}

impl<'a> egui::Widget for ShapesWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if ui.max_rect().width() / ui.max_rect().height() > 1.0 {
            ui.vertical_centered(|ui| self.paint_area_and_shapes(ui))
        } else {
            ui.horizontal_centered(|ui| self.paint_area_and_shapes(ui))
        }
        .response
    }
}

fn cross_shape(size: f32, stroke: egui::Stroke) -> egui::Shape {
    let half_size = size / 2.0;
    egui::Shape::Vec(vec![
        egui::epaint::PathShape::line(
            vec![
                egui::pos2(-half_size, -half_size),
                egui::pos2(half_size, half_size),
            ],
            stroke,
        )
        .into(),
        egui::epaint::PathShape::line(
            vec![
                egui::pos2(half_size, -half_size),
                egui::pos2(-half_size, half_size),
            ],
            stroke,
        )
        .into(),
    ])
}

fn question_mark_shape(size: egui::Vec2, stroke: egui::Stroke) -> egui::Shape {
    let x_left = stroke.width - size.x / 2.0;
    let x_center = 0.0;
    let x_right = size.x / 2.0 - stroke.width;

    let y_top = stroke.width - size.y / 2.0;
    let y_top_bow = size.y * 0.35 - size.y / 2.0;
    let y_lower_bow = size.y * 0.7 - size.y / 2.0;
    let y_end = size.y / 2.0 - stroke.width * 2.0;
    let y_bottom = size.y / 2.0;

    let a = egui::pos2(x_left, y_top_bow);
    let b = egui::pos2(x_center, y_top);
    let c = egui::pos2(x_right, y_top_bow);
    let d = egui::pos2(x_center, y_lower_bow);
    let e = egui::pos2(x_center, y_end);
    let f = egui::pos2(x_center, y_bottom);
    let curve = (d.y - c.y) * 0.5;
    let question_mark: Vec<egui::Shape> = vec![
        epaint::CubicBezierShape::from_points_stroke(
            [a, a - egui::vec2(0.0, curve), b - egui::vec2(curve, 0.0), b],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        )
        .into(),
        epaint::CubicBezierShape::from_points_stroke(
            [b, b + egui::vec2(curve, 0.0), c - egui::vec2(0.0, curve), c],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        )
        .into(),
        epaint::CubicBezierShape::from_points_stroke(
            [c, c + egui::vec2(0.0, curve), d - egui::vec2(0.0, curve), d],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        )
        .into(),
        epaint::PathShape::line(vec![d, e], stroke).into(),
        epaint::PathShape::line(vec![f - egui::vec2(0.0, stroke.width), f], stroke).into(),
    ];
    egui::Shape::Vec(question_mark)
}
