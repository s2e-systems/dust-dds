mod server_transport;

use std::{net::SocketAddr, path::PathBuf};

use dust_dds::{
    dds_async::{
        data_reader::DataReaderAsync, data_reader_listener::DataReaderListener,
        domain_participant_factory::DomainParticipantFactoryAsync,
    },
    infrastructure::{
        configuration::DustDdsConfiguration,
        listener::NO_LISTENER,
        qos::{DataWriterQos, QosKind, TopicQos},
        qos_policy::HistoryQosPolicyKind,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind, SubscriptionMatchedStatus},
    },
    security::plugins::types::DdsSecurityPlugins,
    std_runtime::StdRuntime,
};
use dust_dds_derive::DdsType;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[derive(Debug, DdsType)]
pub struct TextMessage {
    #[dust_dds(key)]
    pub id: u32,
    pub message: String,
}

#[derive(Debug, DdsType)]
pub struct SineWave {
    pub value: f64,
}

struct ReaderListener;

impl DataReaderListener<TextMessage> for ReaderListener {
    fn on_data_available(
        &mut self,
        the_reader: DataReaderAsync<TextMessage>,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            if let Ok(samples) = the_reader
                .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .await
            {
                for sample in samples {
                    if let Some(ref data) = sample.data {
                        println!(
                            "\n========================================\n\
                             Received message on DataReader: (id={}) \"{}\"\n\
                             ========================================",
                            data.id, data.message
                        );
                    }
                }
            }
        }
    }

    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<TextMessage>,
        status: SubscriptionMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            if status.current_count_change > 0 {
                println!(
                    ">> DataReader matched with Browser DataWriter! (total writers: {})",
                    status.current_count
                );
            } else if status.current_count_change < 0 {
                println!(
                    ">> Browser DataWriter disconnected (total writers: {})",
                    status.current_count
                );
            }
        }
    }
}

fn find_web_root() -> PathBuf {
    let candidates = [
        PathBuf::from("bindings/wasm/demo/browser_publisher"),
        PathBuf::from("../browser_publisher"),
        PathBuf::from("demo/browser_publisher"),
    ];
    for c in &candidates {
        if c.join("index.html").exists() {
            return c.clone();
        }
    }
    PathBuf::from("bindings/wasm/demo/browser_publisher")
}

async fn start_http_server(addr: SocketAddr, root: PathBuf) {
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind HTTP server on {addr}: {e}");
            return;
        }
    };

    println!("HTTP Web Server running at: http://{addr}");
    println!("Web root directory: {}\n", root.display());

    while let Ok((mut stream, _)) = listener.accept().await {
        let root = root.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            let n = match stream.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };
            let req = String::from_utf8_lossy(&buf[..n]);
            let first_line = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            let raw_path = if parts.len() >= 2 { parts[1] } else { "/" };
            let rel_path = raw_path
                .trim_start_matches('/')
                .split('?')
                .next()
                .unwrap_or("");
            let file_path = if rel_path.is_empty() {
                root.join("index.html")
            } else {
                root.join(rel_path)
            };

            let (status, content_type, body) = match tokio::fs::read(&file_path).await {
                Ok(bytes) => {
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let mime = match ext {
                        "html" => "text/html",
                        "js" => "application/javascript",
                        "wasm" => "application/wasm",
                        "css" => "text/css",
                        _ => "application/octet-stream",
                    };
                    ("200 OK", mime, bytes)
                }
                Err(_) => ("404 Not Found", "text/plain", b"File not found".to_vec()),
            };

            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                status,
                content_type,
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.write_all(&body).await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn,webrtc_ice=off,webrtc_mdns=off")
            }),
        )
        .init();

    println!("--------------------------------------------------");
    println!("    Dust DDS Native DataReader Application        ");
    println!("--------------------------------------------------");

    // 1. Start HTTP static file server
    let web_root = find_web_root();
    let http_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    tokio::spawn(start_http_server(http_addr, web_root));

    // 2. Start WebRTC/WebSocket Server Transport on 127.0.0.1:9001
    let transport = server_transport::RtpsWebRtcServerTransport::new(
        "127.0.0.1:9001",
        tokio::runtime::Handle::current(),
    );

    let factory = DomainParticipantFactoryAsync::new(
        DustDdsConfiguration::default(),
        StdRuntime::default(),
        transport,
        DdsSecurityPlugins::disabled(),
    );

    // 3. Create DomainParticipant on domain 0
    let participant = factory
        .create_participant(0, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await?;
    println!("DomainParticipant created on Domain 0");

    // 4. Create Topic "TextMessage"
    let topic = participant
        .create_topic::<TextMessage>(
            "TextMessage",
            "TextMessage",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await?;
    println!("Topic 'TextMessage' created");

    // 5. Create Subscriber and DataReader with listener
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await?;

    let listener = ReaderListener;
    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Default,
            Some(listener),
            &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
        )
        .await?;
    println!("DataReader subscribed to 'TextMessage'");

    // 6. Create Topic "SineWave" (without key), with History KeepLast(10)
    let mut sine_topic_qos = TopicQos::default();
    sine_topic_qos.history.kind = HistoryQosPolicyKind::KeepLast(10);
    let sine_topic = participant
        .create_topic::<SineWave>(
            "SineWave",
            "SineWave",
            QosKind::Specific(sine_topic_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .await?;
    println!("Topic 'SineWave' created with History KeepLast(10)");

    // 7. Create Publisher and DataWriter for "SineWave"
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await?;

    let mut sine_writer_qos = DataWriterQos::default();
    sine_writer_qos.history.kind = HistoryQosPolicyKind::KeepLast(10);

    let sine_writer = publisher
        .create_datawriter(
            &sine_topic,
            QosKind::Specific(sine_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .await?;
    println!("DataWriter created for 'SineWave'");

    // 8. Publish a sine wave floating point value every 0.1 seconds (100ms)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        let mut angle: f64 = 0.0;
        loop {
            interval.tick().await;
            let value = angle.sin();
            angle += 0.1;
            if angle >= std::f64::consts::TAU {
                angle -= std::f64::consts::TAU;
            }
            let sample = SineWave { value };
            let _ = sine_writer.write(sample, None).await;
        }
    });
    println!("SineWave publisher active (publishing every 0.1s)");

    println!(
        "\nReady! Open http://127.0.0.1:8080 in your browser to view the sine wave and publish messages."
    );
    println!("Press Ctrl+C to terminate.\n");

    // Keep running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");

    Ok(())
}
