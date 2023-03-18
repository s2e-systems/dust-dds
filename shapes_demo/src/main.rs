#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod shapes_type;

use dust_dds::{
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
    },
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::Duration,
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{
        data_reader::DataReader,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        subscriber::Subscriber,
    },
};
use shapes_type::ShapeType;

use eframe::{
    egui::{self, Rect},
    epaint::{
        pos2, vec2, CircleShape, Color32, PathShape, Pos2, RectShape, Rounding, Shape, Stroke, Vec2,
    },
    Theme,
};

fn main() -> Result<(), eframe::Error> {
    const ICON: &[u8] = include_bytes!("logo.png");
    let icon = image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
        .expect("Failed to open icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 500.0)),
        default_theme: Theme::Light,
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Dust DDS Shapes Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::new())),
    )
}


const PURPLE: Color32 = Color32::from_rgb(128, 0, 128);
const BLUE: Color32 = Color32::BLUE;
const RED: Color32 = Color32::RED;
const GREEN: Color32 = Color32::GREEN;
const YELLOW: Color32 = Color32::YELLOW;
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
const ORANGE: Color32 = Color32::from_rgb(255, 165, 0);

trait NewShape {
    fn new(color: Color32, size: f32, center: Pos2) -> Self;
}
trait ShapeProperties {
    fn color(&self) -> Color32;
    fn size(&self) -> f32;
    fn center(&self) -> Pos2;
}

trait Move {
    fn move_to(&mut self, p: Pos2);
    fn move_by(&mut self, v: Vec2);
}

trait Velocity {
    fn velocity(&self) -> Vec2;
    fn set_velocity(&mut self, v: Vec2);
}

trait AsShapeType {
    fn as_shape_type(&self, offset: Vec2) -> ShapeType;
}

trait AsShape {
    fn as_shape(&self) -> Shape;
}

trait MovingShape: Move + Velocity + ShapeProperties + AsShapeType + AsShape {}
impl<T: Move + Velocity + ShapeProperties + AsShapeType + AsShape> MovingShape for T {}


impl<T: ShapeProperties> AsShapeType for T {
    fn as_shape_type(&self, offset: Vec2) -> ShapeType {
        let center = self.center() - offset;
        let color = match self.color() {
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
            x: center.x as i32,
            y: center.y as i32,
            shapesize: self.size() as i32,
        }
    }
}

#[derive(Clone, Copy)]
enum ShapeKind {
    Circle,
    Triangle,
    Square,
}

impl ShapeKind {
    fn as_str(&self) -> &'_ str {
        match self {
            ShapeKind::Circle => "Circle",
            ShapeKind::Triangle => "Triangle",
            ShapeKind::Square => "Square",
        }
    }
}

struct ShapeWriter {
    writer: DataWriter<ShapeType>,
    shape: Box<dyn MovingShape>,
}
impl ShapeWriter {
    fn write(&self, offset: Vec2) {
        let data = self.shape.as_shape_type(offset);
        self.writer.write(&data, None).expect("writing failed");
        let shape_kind = self.writer.get_topic().unwrap().get_name().unwrap();
        println!("{:?} written {:?}", shape_kind, data);
    }
    fn shape(&mut self) -> Shape {
        self.shape.as_shape()
    }
}

struct MyApp {
    participant: DomainParticipant,
    publisher: Publisher,
    subscriber: Subscriber,
    reader_list: Vec<DataReader<ShapeType>>,
    shape_writer_list: Vec<ShapeWriter>,
    window_open: Option<ShapeKind>,
}

impl MyApp {
    fn new() -> Self {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();
        let publisher = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let subscriber = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .unwrap();

        Self {
            participant,
            publisher,
            subscriber,
            reader_list: vec![],
            shape_writer_list: vec![],
            window_open: None,
        }
    }

    fn create_writer(&mut self, shape_kind: ShapeKind, color: Color32) {
        let topic_name = shape_kind.as_str();

        let topic = self
            .participant
            .create_topic::<ShapeType>(topic_name, QosKind::Default, None, NO_STATUS)
            .unwrap();
        let qos = DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: Duration::new(1, 0),
            },
            ..Default::default()
        };
        let writer = self
            .publisher
            .create_datawriter(&topic, QosKind::Specific(qos), None, NO_STATUS)
            .unwrap();

        let shape: Box<dyn MovingShape> = match shape_kind {
            ShapeKind::Circle => Box::new(MovingCircle::new(color, 30.0, pos2(360.0, 180.0))),
            ShapeKind::Triangle => Box::new(MovingTriangle::new(color, 30.0, pos2(360.0, 180.0))),
            ShapeKind::Square => Box::new(MovingSquare::new(color, 30.0, pos2(360.0, 180.0))),
        };

        let shape_writer = ShapeWriter { writer, shape };
        self.shape_writer_list.push(shape_writer);
    }

    fn create_reader(&mut self, topic_name: &str) {
        let topic = self
            .participant
            .create_topic::<ShapeType>(topic_name, QosKind::Default, None, NO_STATUS)
            .unwrap();
        let qos = DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: Duration::new(1, 0),
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            ..Default::default()
        };
        let reader = self
            .subscriber
            .create_datareader(&topic, QosKind::Specific(qos), None, NO_STATUS)
            .unwrap();
        self.reader_list.push(reader);
    }

    fn read_data(&self, reader: &DataReader<ShapeType>, offset: Vec2) -> Vec<Shape> {
        let shapes_kind = reader.get_topicdescription().unwrap().get_name().unwrap();

        let mut shapes = vec![];
        let mut previous_handle = None;
        let now = std::time::Instant::now();
        while let Ok(samples) = reader.read_next_instance(
            1,
            previous_handle,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            if let Some(sample) = samples.first() {
                previous_handle = Some(sample.sample_info.instance_handle);
                if let Some(data) = &sample.data {
                    println!("{:?} {:?} read {:?}", std::time::Instant::now() - now, shapes_kind, data);
                    let color = match data.color.as_str() {
                        "PURPLE" => PURPLE,
                        "BLUE" => BLUE,
                        "RED" => RED,
                        "GREEN" => GREEN,
                        "YELLOW" => YELLOW,
                        "CYAN" => CYAN,
                        "MAGENTA" => MAGENTA,
                        "ORANGE" => ORANGE,
                        _ => Color32::TEMPORARY_COLOR,
                    };
                    let center = Pos2 {
                        x: data.x as f32,
                        y: data.y as f32,
                    } + offset;

                    let size = data.shapesize as f32;
                    let shape = match shapes_kind.as_str() {
                        "Circle"  => MovingCircle::new(color, size, center).as_shape(),
                        "Triangle" => MovingTriangle::new(color, size, center).as_shape(),
                        "Square" => MovingSquare::new(color, size, center).as_shape(),
                        _ => panic!("Unsupported shape")
                    };
                    shapes.push(shape);
                }
            }
        }
        shapes
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(shape_kind) = self.window_open {
            egui::Window::new("Publish").show(ctx, |ui| {
                    if ui.button("RED").clicked() {
                        self.window_open = None;
                        self.create_writer(shape_kind, RED);
                    }
                    if ui.button("GREEN").clicked() {
                        self.window_open = None;
                        self.create_writer(shape_kind, GREEN);
                    }
                    if ui.button("BLUE").clicked() {
                        self.window_open = None;
                        self.create_writer(shape_kind, BLUE);
                    }
                });
        }

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Publish");
            if ui.button(ShapeKind::Circle.as_str()).clicked() {
                self.window_open = Some(ShapeKind::Circle);
            };
            if ui.button(ShapeKind::Triangle.as_str()).clicked() {
                self.window_open = Some(ShapeKind::Triangle);
            };
            if ui.button(ShapeKind::Square.as_str()).clicked() {
                self.window_open = Some(ShapeKind::Square);
            };
            ui.separator();
            ui.heading("Subscribe");
            if ui.button(ShapeKind::Circle.as_str()).clicked() {
                self.create_reader(ShapeKind::Circle.as_str())
            };
            if ui.button(ShapeKind::Triangle.as_str()).clicked() {
                self.create_reader(ShapeKind::Triangle.as_str())
            };
            if ui.button(ShapeKind::Square.as_str()).clicked() {
                self.create_reader(ShapeKind::Square.as_str())
            };
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rect = Rect::from_center_size(ui.max_rect().center(), vec2(235.0, 265.0));
            painter.rect_filled(rect, Rounding::none(), Color32::WHITE);
            let offset = rect.left_top().to_vec2();
            for shape_writer in &mut self.shape_writer_list {
                shape_writer.write(offset);
                move_within_rect(shape_writer.shape.as_mut(), rect);
                painter.add(shape_writer.shape());
            }
            for reader in &self.reader_list {
                for circle in self.read_data(reader, offset) {
                    painter.add(circle);
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        });
    }
}

fn move_within_rect(object: &mut dyn MovingShape, rect: Rect) {
    let radius = object.size() / 2.0;
    let left = object.center().x - radius;
    let right = object.center().x + radius;
    let top = object.center().y - radius;
    let bottom = object.center().y + radius;
    let normal = if left < rect.left() {
        object.move_to(pos2(rect.left() + radius, object.center().y));
        Some(vec2(1.0, 0.0))
    } else if right > rect.right() {
        object.move_to(pos2(rect.right() - radius, object.center().y));
        Some(vec2(-1.0, 0.0))
    } else if top < rect.top() {
        object.move_to(pos2(object.center().x, rect.top() + radius));
        Some(vec2(0.0, 1.0))
    } else if bottom > rect.bottom() {
        object.move_to(pos2(object.center().x, rect.bottom() - radius));
        Some(vec2(0.0, -1.0))
    } else {
        None
    };
    if let Some(normal) = normal {
        // reflect motion in respect to normal of surface
        object.set_velocity(object.velocity() - 2.0 * (object.velocity() * normal) * normal);
    }
    object.move_by(object.velocity());
}


struct MovingSquare {
    velocity: Vec2,
    shape: RectShape,
}
impl AsShape for MovingSquare {
    fn as_shape(&self) -> Shape {
        self.shape.into()
    }
}
impl NewShape for MovingSquare {
    fn new(color: Color32, size: f32, center: Pos2) -> Self {
        Self {
            velocity: vec2(1.5, 1.5),
            shape: RectShape {
                rect: Rect::from_center_size(center, vec2(size, size)),
                rounding: Rounding::none(),
                fill: color,
                stroke: Stroke::NONE,
            },
        }
    }
}
impl ShapeProperties for MovingSquare {
    fn color(&self) -> Color32 {
        self.shape.fill
    }

    fn size(&self) -> f32 {
        self.shape.rect.width()
    }

    fn center(&self) -> Pos2 {
        self.shape.rect.center()
    }
}

impl Move for MovingSquare {
    fn move_to(&mut self, p: Pos2) {
        self.shape.rect.set_center(p);
    }

    fn move_by(&mut self, v: Vec2) {
        self.shape.rect.set_center(self.shape.rect.center() + v);
    }
}
impl Velocity for MovingSquare {
    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn set_velocity(&mut self, v: Vec2) {
        self.velocity = v;
    }
}

#[derive(Clone)]

struct MovingTriangle {
    velocity: Vec2,
    shape: PathShape,
    size: f32,
}
impl AsShape for MovingTriangle {
    fn as_shape(&self) -> Shape {
        self.shape.clone().into()
    }
}
impl NewShape for MovingTriangle {
    fn new(color: Color32, size: f32, center: Pos2) -> Self {
        let triangle = PathShape {
            points: vec![
                center + vec2(0.0, -size / 2.0),
                center + vec2(-size / 2.0, size / 2.0),
                center + vec2(size / 2.0, size / 2.0),
            ],
            closed: true,
            fill: color,
            stroke: Stroke::NONE,
        };

        Self {
            velocity: vec2(4.0, 4.0),
            shape: triangle,
            size,
        }
    }
}
impl ShapeProperties for MovingTriangle {
    fn color(&self) -> Color32 {
        self.shape.fill
    }

    fn size(&self) -> f32 {
        self.size
    }

    fn center(&self) -> Pos2 {
        pos2(
            self.shape.points[0].x,
            self.shape.points[0].y + self.size / 2.0,
        )
    }
}

impl Move for MovingTriangle {
    fn move_to(&mut self, p: Pos2) {
        self.shape.points = vec![
            p + vec2(0.0, -self.size / 2.0),
            p + vec2(-self.size / 2.0, self.size / 2.0),
            p + vec2(self.size / 2.0, self.size / 2.0),
        ];
    }

    fn move_by(&mut self, v: Vec2) {
        for point in &mut self.shape.points {
            *point += v;
        }
    }
}

impl Velocity for MovingTriangle {
    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn set_velocity(&mut self, v: Vec2) {
        self.velocity = v;
    }
}

#[derive(Clone)]
struct MovingCircle {
    velocity: Vec2,
    shape: CircleShape,
}
impl AsShape for MovingCircle {
    fn as_shape(&self) -> Shape {
        self.shape.into()
    }
}
impl NewShape for MovingCircle {
    fn new(color: Color32, size: f32, center: Pos2) -> Self {
        let circle = CircleShape {
            center,
            radius: size / 2.0,
            fill: color,
            stroke: Stroke::NONE,
        };
        Self {
            shape: circle,
            velocity: vec2(2.0, 2.0),
        }
    }
}
impl ShapeProperties for MovingCircle {
    fn color(&self) -> Color32 {
        self.shape.fill
    }
    fn size(&self) -> f32 {
        self.shape.radius * 2.0
    }
    fn center(&self) -> Pos2 {
        self.shape.center
    }
}

impl Move for MovingCircle {
    fn move_to(&mut self, p: Pos2) {
        self.shape.center = p;
    }

    fn move_by(&mut self, v: Vec2) {
        self.shape.center += v;
    }
}

impl Velocity for MovingCircle {
    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn set_velocity(&mut self, v: Vec2) {
        self.velocity = v;
    }
}
