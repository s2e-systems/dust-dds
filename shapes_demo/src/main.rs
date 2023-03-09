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
        time::{Duration, DurationKind},
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
    epaint::{pos2, vec2, CircleShape, Color32, Pos2, Rounding, Shape, Stroke, Vec2},
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
    moving_circle: MovingCircle,
    participant: DomainParticipant,
    publisher: Publisher,
    subscriber: Subscriber,
    writers: Vec<DataWriter<ShapeType>>,
    readers: Vec<DataReader<ShapeType>>,
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

        let moving_circle = MovingCircle::new(CircleShape {
            center: pos2(360.0, 40.0),
            radius: 15.0,
            fill: Color32::BLUE,
            stroke: Stroke::NONE,
        });

        Self {
            moving_circle,
            participant,
            publisher,
            subscriber,
            writers: vec![],
            readers: vec![],
        }
    }

    fn create_writer(&mut self, topic_name: &str) {
        let topic = self
            .participant
            .create_topic::<ShapeType>(topic_name, QosKind::Default, None, NO_STATUS)
            .unwrap();
        let qos = DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
            },
            ..Default::default()
        };
        let writer = self
            .publisher
            .create_datawriter(&topic, QosKind::Specific(qos), None, NO_STATUS)
            .unwrap();
        self.writers.push(writer);
    }

    fn create_reader(&mut self, topic_name: &str) {
        let topic = self
            .participant
            .create_topic::<ShapeType>(topic_name, QosKind::Default, None, NO_STATUS)
            .unwrap();
        let qos = DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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

    fn write_circle_data(&self, circle: &CircleShape, offset: &Pos2) {
        if let Some(writer) = self.writers.first() {
            let color = "BLUE".to_string();
            let x = (circle.center.x - offset.x) as i32;
            let y = (circle.center.y - offset.y) as i32;
            let shapesize = (circle.radius * 2.0) as i32;
            let data = ShapeType {
                color,
                x,
                y,
                shapesize,
            };
            writer.write(&data, None).expect("writing failed");
            println!("written {:?}", data);
        }
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
            if ui.button("Circle").clicked() {
                self.create_writer("Circle");
            };
            ui.separator();
            ui.heading("Subscribe");
            if ui.button("Circle").clicked() {
                self.create_reader("Circle")
            };
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rect = Rect::from_center_size(ui.max_rect().center(), vec2(235.0, 265.0));
            painter.rect_filled(rect, Rounding::none(), Color32::WHITE);

            let offset = &rect.left_top();
            if let Some(_writer) = self.writers.first() {
                self.moving_circle.move_within_rect(rect);
                self.write_circle_data(&self.moving_circle.circle, offset);
                painter.add(self.moving_circle.clone());
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

#[derive(Clone)]
struct MovingCircle {
    velocity: Vec2,
    circle: CircleShape,
}

impl MovingCircle {
    fn new(circle: CircleShape) -> Self {
        Self {
            circle,
            velocity: vec2(2.0, 2.0),
        }
    }
    fn left(&self) -> f32 {
        self.circle.center.x - self.circle.radius
    }
    fn right(&self) -> f32 {
        self.circle.center.x + self.circle.radius
    }
    fn top(&self) -> f32 {
        self.circle.center.y - self.circle.radius
    }
    fn bottom(&self) -> f32 {
        self.circle.center.y + self.circle.radius
    }

    fn move_within_rect(&mut self, rect: Rect) {
        let normal = if self.left() < rect.left() {
            self.circle.center.x = rect.left() + self.circle.radius;
            Some(vec2(1.0, 0.0))
        } else if self.right() > rect.right() {
            self.circle.center.x = rect.right() - self.circle.radius;
            Some(vec2(-1.0, 0.0))
        } else if self.top() < rect.top() {
            self.circle.center.y = rect.top() + self.circle.radius;
            Some(vec2(0.0, 1.0))
        } else if self.bottom() > rect.bottom() {
            self.circle.center.y = rect.bottom() - self.circle.radius;
            Some(vec2(0.0, -1.0))
        } else {
            None
        };
        if let Some(normal) = normal {
            // reflect motion in respect to normal of surface
            self.velocity = self.velocity - 2.0 * (self.velocity * normal) * normal;
        }
        self.circle.center += self.velocity;
    }
}

impl From<MovingCircle> for Shape {
    fn from(v: MovingCircle) -> Self {
        v.circle.into()
    }
}
