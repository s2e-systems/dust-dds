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

struct MyApp {
    participant: DomainParticipant,
    publisher: Publisher,
    subscriber: Subscriber,
    readers: Vec<DataReader<ShapeType>>,
    shape_writer_list: Vec<ShapeWriter>,
}

trait IntoShapeType {
    fn into_shape_type(&self) -> ShapeType;
}

impl IntoShapeType for MovingTriangle {
    fn into_shape_type(&self) -> ShapeType {
        let color = match self.triangle.fill {
            Color32::RED => "RED",
            Color32::BLUE => "BLUE",
            Color32::GREEN => "GREEN",
            _ => panic!("Color invalid"),
        }
        .to_string();
        let center = self.bb().center();
        ShapeType {
            color,
            x: center.x as i32,
            y: center.y as i32,
            shapesize: self.size as i32,
        }
    }
}
impl IntoShapeType for MovingCircle {
    fn into_shape_type(&self) -> ShapeType {
        let color = match self.circle.fill {
            Color32::RED => "RED",
            Color32::BLUE => "BLUE",
            Color32::GREEN => "GREEN",
            _ => panic!("Color invalid"),
        }
        .to_string();
        let center = self.bb().center();
        ShapeType {
            color,
            x: center.x as i32,
            y: center.y as i32,
            shapesize: self.bb().width() as i32,
        }
    }
}

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

enum MovingShapeKind {
    Circle(MovingCircle),
    Triangle(MovingTriangle),
    Square(MovingSquare),
}

impl MovingShapeKind {
    fn into_shape_type(&self) -> ShapeType {
        match self {
            MovingShapeKind::Circle(shape) => shape.into_shape_type(),
            MovingShapeKind::Triangle(shape) => shape.into_shape_type(),
            MovingShapeKind::Square(shape) => shape.into_shape_type(),
        }
    }

    fn moved_shape(&mut self) -> Shape {
        match self {
            MovingShapeKind::Circle(shape) => shape.circle.into(),
            MovingShapeKind::Triangle(shape) => shape.triangle.clone().into(),
            MovingShapeKind::Square(shape) => shape.shape.into(),
        }
    }
}

impl BoundingBox for MovingShapeKind {
    fn bb(&self) -> Rect {
        match self {
            MovingShapeKind::Circle(shape) => shape.bb(),
            MovingShapeKind::Triangle(shape) => shape.bb(),
            MovingShapeKind::Square(shape) => shape.bb(),
        }
    }
}

impl Move for MovingShapeKind {
    fn move_to(&mut self, p: Pos2) {
        match self {
            MovingShapeKind::Circle(shape) => shape.move_to(p),
            MovingShapeKind::Triangle(shape) => shape.move_to(p),
            MovingShapeKind::Square(shape) => shape.move_to(p),
        }
    }

    fn move_by(&mut self, v: Vec2) {
        match self {
            MovingShapeKind::Circle(shape) => shape.move_by(v),
            MovingShapeKind::Triangle(shape) => shape.move_by(v),
            MovingShapeKind::Square(shape) => shape.move_by(v),
        }
    }
}

impl Velocity for MovingShapeKind {
    fn velocity(&self) -> Vec2 {
        match self {
            MovingShapeKind::Circle(shape) => shape.velocity(),
            MovingShapeKind::Triangle(shape) => shape.velocity(),
            MovingShapeKind::Square(shape) => shape.velocity(),
        }
    }

    fn set_velocity(&mut self, v: Vec2) {
        match self {
            MovingShapeKind::Circle(shape) => shape.set_velocity(v),
            MovingShapeKind::Triangle(shape) => shape.set_velocity(v),
            MovingShapeKind::Square(shape) => shape.set_velocity(v),
        }
    }
}

struct ShapeWriter {
    writer: DataWriter<ShapeType>,
    shape: MovingShapeKind,
}
impl ShapeWriter {
    fn write(&self) {
        let data = self.shape.into_shape_type();
        self.writer.write(&data, None).expect("writing failed");
        println!("written {:?}", data);
    }
    fn moved_shape(&mut self) -> Shape {
        self.shape.moved_shape().into()
    }
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
            readers: vec![],
            shape_writer_list: vec![],
        }
    }

    fn create_writer(&mut self, shape_kind: ShapeKind) {
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

        let shape = match shape_kind {
            ShapeKind::Circle => MovingShapeKind::Circle(MovingCircle::new(CircleShape {
                center: pos2(360.0, 180.0),
                radius: 15.0,
                fill: Color32::BLUE,
                stroke: Stroke::NONE,
            })),
            ShapeKind::Triangle => MovingShapeKind::Triangle(MovingTriangle::new()),
            ShapeKind::Square => MovingShapeKind::Square(MovingSquare::new(RectShape {
                rect: Rect::from_center_size(pos2(360.0, 180.0), vec2(30.0, 30.0)),
                rounding: Rounding::none(),
                fill: Color32::BLUE,
                stroke: Stroke::NONE,
            })),
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
        self.readers.push(reader);
    }

    fn read_circle_data(&self, reader: &DataReader<ShapeType>, offset: &Pos2) -> Vec<CircleShape> {
        let mut shapes = vec![];
        let mut previous_handle = None;
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
                    println!("read {:?}", data);
                    let fill = match data.color.as_str() {
                        "PURPLE" => Color32::from_rgb(128, 0, 128),
                        "BLUE" => Color32::BLUE,
                        "RED" => Color32::RED,
                        "GREEN" => Color32::GREEN,
                        "YELLOW" => Color32::YELLOW,
                        "CYAN" => Color32::from_rgb(0, 255, 255),
                        "MAGENTA" => Color32::from_rgb(255, 0, 255),
                        "ORANGE" => Color32::from_rgb(255, 165, 0),
                        _ => Color32::TEMPORARY_COLOR,
                    };
                    let stroke = Stroke::new(3.0, Color32::RED);
                    let center = Pos2 {
                        x: data.x as f32 + offset.x,
                        y: data.y as f32 + offset.y,
                    };

                    let radius = data.shapesize as f32 / 2.0;
                    shapes.push(CircleShape {
                        center,
                        fill,
                        radius,
                        stroke,
                    });
                }
            }
        }
        shapes
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Publish");
            if ui.button(ShapeKind::Circle.as_str()).clicked() {
                self.create_writer(ShapeKind::Circle);
            };
            if ui.button(ShapeKind::Triangle.as_str()).clicked() {
                self.create_writer(ShapeKind::Triangle);
            };
            if ui.button(ShapeKind::Square.as_str()).clicked() {
                self.create_writer(ShapeKind::Square);
            };
            ui.separator();
            ui.heading("Subscribe");
            if ui.button(ShapeKind::Circle.as_str()).clicked() {
                self.create_reader(ShapeKind::Circle.as_str())
            };
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rect = Rect::from_center_size(ui.max_rect().center(), vec2(235.0, 265.0));
            painter.rect_filled(rect, Rounding::none(), Color32::WHITE);
            let offset = &rect.left_top();
            for shape_writer in &mut self.shape_writer_list {
                shape_writer.write();
                move_within_rect(&mut shape_writer.shape, rect);
                painter.add(shape_writer.moved_shape());
            }
            for reader in &self.readers {
                for circle in self.read_circle_data(reader, offset) {
                    painter.add(circle);
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        });
    }
}

fn move_within_rect<T>(object: &mut T, rect: Rect)
where
    T: BoundingBox + Move + Velocity,
{
    let normal = if object.bb().left() < rect.left() {
        object.move_to(pos2(
            rect.left() + object.bb().width() / 2.0,
            object.bb().center().y,
        ));
        Some(vec2(1.0, 0.0))
    } else if object.bb().right() > rect.right() {
        object.move_to(pos2(
            rect.right() - object.bb().width() / 2.0,
            object.bb().center().y,
        ));
        Some(vec2(-1.0, 0.0))
    } else if object.bb().top() < rect.top() {
        object.move_to(pos2(
            object.bb().center().x,
            rect.top() + object.bb().height() / 2.0,
        ));
        Some(vec2(0.0, 1.0))
    } else if object.bb().bottom() > rect.bottom() {
        object.move_to(pos2(
            object.bb().center().x,
            rect.bottom() - object.bb().height() / 2.0,
        ));
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

trait BoundingBox {
    fn bb(&self) -> Rect;
}

trait Move {
    fn move_to(&mut self, p: Pos2);
    fn move_by(&mut self, v: Vec2);
}

struct MovingSquare {
    velocity: Vec2,
    shape: RectShape,
}

impl MovingSquare {
    fn new(shape: RectShape) -> Self {
        Self {
            velocity: vec2(1.5, 1.5),
            shape,
        }
    }
}
impl BoundingBox for MovingSquare {
    fn bb(&self) -> Rect {
        self.shape.rect
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
impl IntoShapeType for MovingSquare {
    fn into_shape_type(&self) -> ShapeType {
        let color = match self.shape.fill {
            Color32::RED => "RED",
            Color32::BLUE => "BLUE",
            Color32::GREEN => "GREEN",
            _ => panic!("Color invalid"),
        }
        .to_string();
        let center = self.bb().center();
        ShapeType {
            color,
            x: center.x as i32,
            y: center.y as i32,
            shapesize: self.bb().width() as i32,
        }
    }
}

#[derive(Clone)]

struct MovingTriangle {
    velocity: Vec2,
    triangle: PathShape,
    size: f32,
}

impl BoundingBox for MovingTriangle {
    fn bb(&self) -> Rect {
        Rect {
            min: pos2(self.triangle.points[1].x, self.triangle.points[0].y),
            max: pos2(self.triangle.points[2].x, self.triangle.points[2].y),
        }
    }
}
impl Move for MovingTriangle {
    fn move_to(&mut self, p: Pos2) {
        self.triangle.points = vec![
            p + vec2(0.0, -self.size / 2.0),
            p + vec2(-self.size / 2.0, self.size / 2.0),
            p + vec2(self.size / 2.0, self.size / 2.0),
        ];
    }

    fn move_by(&mut self, v: Vec2) {
        for point in &mut self.triangle.points {
            *point += v;
        }
    }
}
impl MovingTriangle {
    fn new() -> Self {
        let size = 30.0;
        let triangle = PathShape { points: vec![
            pos2(0.0, -size / 2.0),
            pos2(-size / 2.0, size / 2.0),
            pos2(size / 2.0, size / 2.0),
        ], closed: true, fill: Color32::BLUE, stroke: Stroke::NONE };

        Self {
            velocity: vec2(4.0, 4.0),
            triangle,
            size,
        }
    }
}

impl From<MovingTriangle> for Shape {
    fn from(v: MovingTriangle) -> Self {
        v.triangle.into()
    }
}

trait Velocity {
    fn velocity(&self) -> Vec2;
    fn set_velocity(&mut self, v: Vec2);
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
    circle: CircleShape,
}
impl BoundingBox for MovingCircle {
    fn bb(&self) -> Rect {
        Rect {
            min: self.circle.center - vec2(self.circle.radius, self.circle.radius),
            max: self.circle.center + vec2(self.circle.radius, self.circle.radius),
        }
    }
}

impl Move for MovingCircle {
    fn move_to(&mut self, p: Pos2) {
        self.circle.center = p;
    }

    fn move_by(&mut self, v: Vec2) {
        self.circle.center += v;
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

impl MovingCircle {
    fn new(circle: CircleShape) -> Self {
        Self {
            circle,
            velocity: vec2(2.0, 2.0),
        }
    }
}

impl From<MovingCircle> for Shape {
    fn from(v: MovingCircle) -> Self {
        v.circle.into()
    }
}
