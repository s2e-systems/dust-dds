pub mod shapes_type {
    include!(concat!(env!("OUT_DIR"), "/idl/shapes_type.rs"));
}

use self::shapes_type::ShapeType;
use super::shapes_widget::{GuiShape, MovingShapeObject, ShapesWidget};
use crate::shapes_widget::ShapesMarker;
use dust_dds::{
    dds_async::data_writer::DataWriterAsync,
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
    },
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DestinationOrderQosPolicyKind,
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            WriterDataLifecycleQosPolicy,
        },
        sample_info::{InstanceStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{OfferedDeadlineMissedStatus, StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
    },
    listener::NO_LISTENER,
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener, publisher::Publisher,
    },
    std_runtime::StdRuntime,
    subscription::{data_reader::DataReader, subscriber::Subscriber},
};
use eframe::{
    egui::{self},
    epaint::vec2,
};
use std::sync::{Arc, Mutex};

impl From<InstanceStateKind> for ShapesMarker {
    fn from(value: InstanceStateKind) -> Self {
        match value {
            InstanceStateKind::NotAliveNoWriters => Self::QuestionMark,
            InstanceStateKind::NotAliveDisposed => Self::Cross,
            InstanceStateKind::Alive => Self::Blank,
        }
    }
}

struct ShapeWriter {
    writer: DataWriter<StdRuntime, ShapeType>,
    shape: Arc<Mutex<MovingShapeObject>>,
}
impl ShapeWriter {
    fn color(&self) -> String {
        self.shape
            .lock()
            .expect("shape_lockable")
            .gui_shape()
            .as_shape_type()
            .color
            .clone()
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
    participant: DomainParticipant<StdRuntime>,
    publisher: Publisher<StdRuntime>,
    subscriber: Subscriber<StdRuntime>,
    reader_list: Vec<DataReader<StdRuntime, ShapeType>>,
    writer_list: Vec<ShapeWriter>,
    time: f64,
    is_reliable_reader: bool,
    publish_widget: Option<PublishWidget>,
}

impl Default for ShapesDemoApp {
    fn default() -> Self {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();
        let publisher = participant
            .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();
        let subscriber = participant
            .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
            .unwrap();

        Self {
            participant,
            publisher,
            subscriber,
            reader_list: vec![],
            writer_list: vec![],
            time: 0.0,
            is_reliable_reader: false,
            publish_widget: None,
        }
    }
}

impl ShapesDemoApp {
    fn create_writer(&mut self, shape_kind: String, color: &str, is_reliable: bool) {
        let topic_name = shape_kind.as_str();
        let topic = if let Some(topic) = self
            .participant
            .lookup_topicdescription(topic_name)
            .unwrap()
        {
            topic
        } else {
            self.participant
                .create_topic::<ShapeType>(
                    topic_name,
                    "ShapeType",
                    QosKind::Default,
                    NO_LISTENER,
                    NO_STATUS,
                )
                .unwrap()
        };
        let writer_data_lifecycle = WriterDataLifecycleQosPolicy {
            autodispose_unregistered_instances: false,
        };
        let reliability = if is_reliable {
            ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            }
        } else {
            ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Infinite,
            }
        };

        let qos = DataWriterQos {
            reliability,
            destination_order: DestinationOrderQosPolicy {
                kind: DestinationOrderQosPolicyKind::BySourceTimestamp,
            },
            deadline: DeadlineQosPolicy {
                period: DurationKind::Finite(Duration::new(0, 1_000_000_000)),
            },
            writer_data_lifecycle,
            ..Default::default()
        };

        let writer = self
            .publisher
            .create_datawriter(&topic, QosKind::Specific(qos), NO_LISTENER, NO_STATUS)
            .unwrap();

        let velocity = vec2(30.0, 20.0);
        let shape_type = ShapeType {
            color: color.to_string(),
            x: 100,
            y: 80,
            shapesize: 30,
        };
        writer.write(&shape_type, None).ok();

        let shape = Arc::new(Mutex::new(MovingShapeObject::new(
            GuiShape::from_shape_type(shape_kind, &shape_type, 255, ShapesMarker::Blank),
            velocity,
        )));
        struct Listener {
            shape: Arc<Mutex<MovingShapeObject>>,
            start: std::time::Instant
        }

        impl<R: dust_dds::runtime::DdsRuntime> DataWriterListener<R, ShapeType> for Listener {
            async fn on_offered_deadline_missed(
                &mut self,
                the_writer: DataWriterAsync<R, ShapeType>,
                status: OfferedDeadlineMissedStatus,
            ) {
                let s = the_writer.get_offered_deadline_missed_status().await;
                println!("[{:?}] {:?} {:?}", self.start.elapsed(), status, s);
                let data = self
                    .shape
                    .lock()
                    .expect("shape_lockable")
                    .gui_shape()
                    .as_shape_type();
                the_writer.write(&data, None).await.ok();
            }
        }
        writer
            .set_listener(
                Some(Listener {
                    shape: shape.clone(),
                    start: std::time::Instant::now()
                }),
                &[StatusKind::OfferedDeadlineMissed],
            )
            .unwrap();

        let shape_writer = ShapeWriter { writer, shape };

        self.writer_list.push(shape_writer);
    }

    fn create_reader(&mut self, topic_name: &str, is_reliable: bool) {
        let topic = if let Some(topic) = self
            .participant
            .lookup_topicdescription(topic_name)
            .unwrap()
        {
            topic
        } else {
            self.participant
                .create_topic::<ShapeType>(
                    topic_name,
                    "ShapeType",
                    QosKind::Default,
                    NO_LISTENER,
                    NO_STATUS,
                )
                .unwrap()
        };
        let history_kind = HistoryQosPolicyKind::KeepLast(6);
        let qos = if is_reliable {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy { kind: history_kind },
                destination_order: DestinationOrderQosPolicy {
                    kind: DestinationOrderQosPolicyKind::BySourceTimestamp,
                },
                ..Default::default()
            }
        } else {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy { kind: history_kind },
                ..Default::default()
            }
        };
        let reader = self
            .subscriber
            .create_datareader(&topic, QosKind::Specific(qos), NO_LISTENER, NO_STATUS)
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
        let mut rate = 50;
        ui.add(egui::Slider::new(&mut rate, 5..=500));

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
        egui_extras::install_image_loaders(ctx);

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
                .max_width(180.0)
                .resizable(false)
                .show(ctx, |ui| self.menu_panel(ui));
            egui::TopBottomPanel::bottom("reader_writer_list")
                .min_height(100.0)
                .show(ctx, |ui| {
                    egui::Grid::new("reader_writer_list_grid")
                        .num_columns(6)
                        .min_col_width(15.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("");
                            ui.label("");
                            ui.label("");
                            ui.label("Topic");
                            ui.label("Color");
                            ui.label("Reliability");
                            ui.end_row();
                            self.writer_list.retain(|shape_writer| {
                                let dispose_button = ui
                                    .button("D")
                                    .on_hover_text("Dispose instance and delete data writer");
                                let unregister_button = ui
                                    .button("U")
                                    .on_hover_text("Unregister instance and delete data writer");
                                ui.label("writer");
                                ui.label(shape_writer.writer.get_topic().get_name());
                                ui.label(shape_writer.color());
                                ui.label(reliability_kind(
                                    &shape_writer.writer.get_qos().unwrap().reliability.kind,
                                ));
                                ui.end_row();
                                let instance = &ShapeType {
                                    color: shape_writer.color(),
                                    x: Default::default(),
                                    y: Default::default(),
                                    shapesize: Default::default(),
                                };
                                if dispose_button.clicked() {
                                    shape_writer.writer.dispose(instance, None).unwrap();
                                };
                                if unregister_button.clicked() {
                                    shape_writer
                                        .writer
                                        .unregister_instance(instance, None)
                                        .unwrap();
                                };
                                if dispose_button.clicked() || unregister_button.clicked() {
                                    shape_writer
                                        .writer
                                        .get_publisher()
                                        .delete_datawriter(&shape_writer.writer)
                                        .is_err()
                                } else {
                                    true
                                }
                            });

                            ui.end_row();
                            self.reader_list.retain(|reader| {
                                let delete_button =
                                    ui.button("D").on_hover_text("Delete data reader");
                                ui.label("");
                                ui.label("reader");
                                ui.label(reader.get_topicdescription().get_name());
                                ui.label("*");
                                ui.label(reliability_kind(
                                    &reader.get_qos().unwrap().reliability.kind,
                                ));
                                ui.end_row();
                                if delete_button.clicked() {
                                    reader.get_subscriber().delete_datareader(reader).is_err()
                                } else {
                                    true
                                }
                            });
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
                    100,
                    previous_handle,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                ) {
                    let mut alpha = 50;
                    let alpha_step = (255 - alpha) / samples.len() as u8;
                    for sample in samples.iter() {
                        previous_handle = Some(sample.sample_info().instance_handle);
                        if let Ok(shape_type) = sample.data() {
                            let shape = GuiShape::from_shape_type(
                                kind.clone(),
                                &shape_type,
                                alpha,
                                sample.sample_info().instance_state.into(),
                            );
                            shape_list.push(shape);
                        }
                        alpha += alpha_step;
                    }
                }
            }

            let time = ui.input(|i| i.time);
            let time_delta = (time - self.time) as f32;
            self.time = time;
            for writer in self.writer_list.iter_mut() {
                writer
                    .shape
                    .lock()
                    .expect("shape_lockable")
                    .move_within_rect(rect_size, time_delta);
                shape_list.push(
                    writer
                        .shape
                        .lock()
                        .expect("shape_lockable")
                        .gui_shape()
                        .clone(),
                );
            }
            ui.add(ShapesWidget::new(rect_size, shape_list.as_slice()));

            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        });
    }
}
