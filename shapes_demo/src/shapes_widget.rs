use eframe::egui::{self};

use super::app::shapes_type::ShapeType;

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
}

impl GuiShape {
    pub fn from_shape_type(kind: String, shape_type: &ShapeType) -> Self {
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
        Self {
            kind,
            color,
            position: egui::pos2(shape_type.x as f32, shape_type.y as f32),
            size: shape_type.shapesize as f32,
        }
    }

    pub fn as_shape_type(&self) -> ShapeType {
        let color = match self.color {
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

        match self.kind.as_str() {
            "Circle" => egui::epaint::CircleShape {
                center: position,
                radius: size / 2.0,
                fill: self.color,
                stroke,
            }
            .into(),
            "Triangle" => egui::epaint::PathShape {
                points: vec![
                    position + egui::vec2(0.0, -size / 2.0),
                    position + egui::vec2(-size / 2.0, size / 2.0),
                    position + egui::vec2(size / 2.0, size / 2.0),
                ],
                closed: true,
                fill: self.color,
                stroke,
            }
            .into(),
            "Square" => egui::epaint::RectShape::new(
                egui::Rect::from_center_size(position, egui::epaint::vec2(size, size)),
                egui::Rounding::ZERO,
                self.color,
                stroke,
            )
            .into(),
            _ => panic!("shape kind not valid")
        }
    }

}

#[derive(Clone)]
pub struct MovingShapeObject {
    gui_shape: GuiShape,
    velocity: egui::Vec2,
}

impl MovingShapeObject {
    pub fn new(shape: GuiShape, velocity: egui::Vec2) -> Self {
        Self { gui_shape: shape, velocity }
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
