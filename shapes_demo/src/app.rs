pub mod shapes_type {
    include!("../target/idl/shapes_type.rs");
}

use self::shapes_type::ShapeType;
use super::shapes_widget::{GuiShape, MovingShapeObject, ShapesWidget};
use dust_dds::{
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
    },
    infrastructure::{
        listeners::NoOpListener,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::DurationKind,
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{
        data_reader::DataReader,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        subscriber::Subscriber,
    },
};
use eframe::{
    egui::{self},
    epaint::vec2,
};
use std::sync::{Arc, Mutex};

struct ShapeWriter {
    writer: DataWriter<ShapeType>,
    shape: MovingShapeObject,
}
impl ShapeWriter {
    fn write(&self) {
        let data = self.shape.gui_shape().as_shape_type();
        self.writer.write(&data, None).expect("writing failed");
    }
    fn color(&self) -> String {
        self.shape.gui_shape().as_shape_type().color.clone()
    }
}
fn reliability_kind(kind: &ReliabilityQosPolicyKind) -> &'static str {
    match kind {
        ReliabilityQosPolicyKind::BestEffort => "Best effort",
        ReliabilityQosPolicyKind::Reliable => "Reliable",
    }
}

#[derive(Clone)]
struct PublishWidget {
    selected_shape: String,
    is_reliable: bool,
    selected_color: Option<String>,
}

impl PublishWidget {
    fn new(selected_shape: String) -> Self {
        Self {
            selected_shape,
            is_reliable: false,
            selected_color: None,
        }
    }
    fn add_button(&mut self, ui: &mut egui::Ui, color: &str) {
        if ui.button(color).clicked() {
            self.selected_color = Some(color.to_string());
        }
    }
}

impl egui::Widget for &mut PublishWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.add_button(ui, "PURPLE");
        self.add_button(ui, "BLUE");
        self.add_button(ui, "RED");
        self.add_button(ui, "GREEN");
        self.add_button(ui, "YELLOW");
        self.add_button(ui, "CYAN");
        self.add_button(ui, "MAGENTA");
        self.add_button(ui, "ORANGE");
        ui.checkbox(&mut self.is_reliable, "reliable")
    }
}

pub struct ShapesDemoApp {
    participant: DomainParticipant,
    publisher: Publisher,
    subscriber: Subscriber,
    reader_list: Vec<DataReader<ShapeType>>,
    writer_list: Arc<Mutex<Vec<ShapeWriter>>>,
    time: f64,
    is_reliable_reader: bool,
    publish_widget: Option<PublishWidget>,
    planner: Planner,
}

struct Planner {
    writer_list: Arc<Mutex<Vec<ShapeWriter>>>,
    rate: Arc<Mutex<u64>>,
}

impl Planner {
    fn new(writer_list: Arc<Mutex<Vec<ShapeWriter>>>) -> Self {
        Self {
            writer_list,
            rate: Arc::new(Mutex::new(25)),
        }
    }

    fn start(&mut self) {
        let writer_list_clone = self.writer_list.clone();
        let rate_clone = self.rate.clone();
        std::thread::spawn(move || loop {
            let rate = *rate_clone.lock().unwrap();
            for writer in writer_list_clone.lock().unwrap().iter() {
                writer.write()
            }
            std::thread::sleep(std::time::Duration::from_millis(rate));
        });
    }
}
impl Default for ShapesDemoApp {
    fn default() -> Self {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();
        let publisher = participant
            .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();
        let subscriber = participant
            .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();

        let writer_list = Arc::new(Mutex::new(Vec::new()));
        let mut planner = Planner::new(writer_list.clone());
        planner.start();

        Self {
            participant,
            publisher,
            subscriber,
            reader_list: vec![],
            writer_list,
            time: 0.0,
            is_reliable_reader: false,
            publish_widget: None,
            planner,
        }
    }
}

impl ShapesDemoApp {
    fn create_writer(&mut self, shape_kind: String, color: &str, is_reliable: bool) {
        let topic_name = shape_kind.as_str();

        let topic = self
            .participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        let qos = if is_reliable {
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Infinite,
                },
                ..Default::default()
            }
        } else {
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DurationKind::Infinite,
                },
                ..Default::default()
            }
        };
        let writer = self
            .publisher
            .create_datawriter(
                &topic,
                QosKind::Specific(qos),
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();

        let velocity = vec2(30.0, 20.0);
        let shape_type = &ShapeType {
            color: color.to_string(),
            x: 100,
            y: 80,
            shapesize: 30,
        };

        let shape =
            MovingShapeObject::new(GuiShape::from_shape_type(shape_kind, shape_type), velocity);

        let shape_writer = ShapeWriter { writer, shape };
        self.writer_list.lock().unwrap().push(shape_writer);
    }

    fn create_reader(&mut self, topic_name: &str, is_reliable: bool) {
        let topic = self
            .participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        let qos = if is_reliable {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                ..Default::default()
            }
        } else {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                ..Default::default()
            }
        };
        let reader = self
            .subscriber
            .create_datareader(
                &topic,
                QosKind::Specific(qos),
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        self.reader_list.push(reader);
    }

    fn menu_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Publish");
        if ui.button("Square").clicked() {
            self.publish_widget = Some(PublishWidget::new("Square".to_string()));
        };
        if ui.button("Circle").clicked() {
            self.publish_widget = Some(PublishWidget::new("Circle".to_string()));
        };
        if ui.button("Triangle").clicked() {
            self.publish_widget = Some(PublishWidget::new("Triangle".to_string()));
        };

        ui.separator();
        ui.label("Publish rate [ms]:");
        let mut rate = *self.planner.rate.lock().unwrap();
        ui.add(egui::Slider::new(&mut rate, 5..=500));
        *self.planner.rate.lock().unwrap() = rate;

        ui.separator();
        ui.heading("Subscribe");
        if ui.button("Square").clicked() {
            self.create_reader("Square", self.is_reliable_reader)
        };
        if ui.button("Circle").clicked() {
            self.create_reader("Circle", self.is_reliable_reader)
        };
        if ui.button("Triangle").clicked() {
            self.create_reader("Triangle", self.is_reliable_reader)
        };
        ui.checkbox(&mut self.is_reliable_reader, "reliable");
    }
}

impl eframe::App for ShapesDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(publish_widget) = &mut self.publish_widget {
            let mut open = true;
            egui::Window::new("Publish")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add(publish_widget);
                });
            if !open {
                self.publish_widget = None;
            }
        }
        if let Some(publish_widget) = &self.publish_widget {
            if let Some(color) = &publish_widget.selected_color {
                self.create_writer(
                    publish_widget.selected_shape.clone(),
                    &color.clone(),
                    publish_widget.is_reliable,
                );
                self.publish_widget = None;
            }
        }

        let is_landscape = ctx.screen_rect().aspect_ratio() > 1.0;

        if is_landscape {
            egui::SidePanel::left("menu_panel")
                .max_width(100.0)
                .resizable(false)
                .show(ctx, |ui| self.menu_panel(ui));
            egui::TopBottomPanel::bottom("writer_list")
                .min_height(100.0)
                .show(ctx, |ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(4)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("");
                            ui.label("Topic");
                            ui.label("Color");
                            ui.label("Reliability");
                            ui.end_row();
                            for shape_writer in self.writer_list.lock().unwrap().iter() {
                                ui.label("writer");
                                ui.label(shape_writer.writer.get_topic().get_name());
                                ui.label(shape_writer.color());
                                ui.label(reliability_kind(
                                    &shape_writer.writer.get_qos().unwrap().reliability.kind,
                                ));
                                ui.end_row();
                            }
                            ui.end_row();
                            for reader in self.reader_list.iter() {
                                ui.label("reader");
                                ui.label(reader.get_topicdescription().get_name());
                                ui.label("*");
                                ui.label(reliability_kind(
                                    &reader.get_qos().unwrap().reliability.kind,
                                ));
                                ui.end_row();
                            }
                        })
                });
        } else {
            egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| self.menu_panel(ui));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect_size = egui::vec2(235.0, 265.0);

            let mut shape_list = Vec::new();
            for reader in &self.reader_list {
                let kind = reader.get_topicdescription().get_name();
                let mut previous_handle = None;
                while let Ok(samples) = reader.read_next_instance(
                    1,
                    previous_handle,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                ) {
                    if let Some(sample) = samples.first() {
                        previous_handle = Some(sample.sample_info().instance_handle);
                        if let Ok(shape_type) = sample.data() {
                            let shape = GuiShape::from_shape_type(kind.clone(), &shape_type);
                            shape_list.push(shape);
                        }
                    }
                }
            }

            let time = ui.input(|i| i.time);
            let time_delta = (time - self.time) as f32;
            self.time = time;
            for writer in self.writer_list.lock().unwrap().iter_mut() {
                writer.shape.move_within_rect(rect_size, time_delta);
                shape_list.push(writer.shape.gui_shape().clone());
            }
            ui.add(ShapesWidget::new(rect_size, shape_list.as_slice()));

            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        });
    }
}
