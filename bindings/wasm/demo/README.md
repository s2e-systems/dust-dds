# Dust DDS WebRTC Bidirectional Browser & Native Demo

This demo illustrates bidirectional DDS communication between:
1. **Browser Application**: A web client running in Chrome/Firefox/Safari using `dust_dds_wasm` in pure JavaScript.
2. **Native Application**: A native Rust application using `dust_dds` running in your terminal.

Communication is established via WebRTC DataChannels over RTPS (Real-Time Publish-Subscribe).

---

## Architecture & Data Flows

```text
+------------------------------------+          WebSocket Signaling (ws://127.0.0.1:9001)
|  Browser Application (JavaScript)  |<--------------------------------------------------->+
|                                    |                                                     |
|  1. TextMessage DataWriter         |           WebRTC RTCDataChannel ("browser-data")    |
|     - Writes user messages         |===================================================> |
|                                    |                                                     |
|  2. SineWave DataReader            |           WebRTC RTCDataChannel ("browser-data")    |
|     - History QoS: KeepLast(10)    | <===================================================|
|     - Plots samples on <canvas>    |                                                     |
+------------------------------------+                                                     |
                                                                                           |
                                                                                           v
                                                                            +-------------------------------------+
                                                                            | Native Application (Rust)           |
                                                                            |                                     |
                                                                            | 1. TextMessage DataReader           |
                                                                            |    - Prints received messages       |
                                                                            |                                     |
                                                                            | 2. SineWave DataWriter              |
                                                                            |    - History QoS: KeepLast(10)      |
                                                                            |    - Publishes sin(θ) every 0.1s    |
                                                                            |                                     |
                                                                            | 3. RtpsWebRtcServerTransport        |
                                                                            | 4. Embedded HTTP Server (port 8080) |
                                                                            +-------------------------------------+
```

---

## Topics and Data Models

### 1. Topic `"TextMessage"` (Keyed)
- **Native Rust (`dust_dds`)**:
  ```rust
  #[derive(Debug, DdsType)]
  pub struct TextMessage {
      #[dust_dds(key)]
      pub id: u32,
      pub message: String,
  }
  ```

- **Browser JavaScript (`dust_dds_wasm`)**:
  ```javascript
  const idType = DynamicTypeBuilderFactory.getPrimitiveType(TypeKind.Uint32);
  const idMember = new MemberDescriptor("id", idType, 0, true, false);

  const messageType = DynamicTypeBuilderFactory.createStringType();
  const messageMember = new MemberDescriptor("message", messageType, 1, false, false);

  const builder = DynamicTypeBuilderFactory.createStructureType("TextMessage");
  builder.addMember(idMember);
  builder.addMember(messageMember);
  const textMsgType = builder.build();
  ```

### 2. Topic `"SineWave"` (No Key, History QoS `KeepLast(10)`)
- **Native Rust (`dust_dds`)**:
  ```rust
  #[derive(Debug, DdsType)]
  pub struct SineWave {
      pub value: f64,
  }

  let mut sine_topic_qos = TopicQos::default();
  sine_topic_qos.history.kind = HistoryQosPolicyKind::KeepLast(10);
  ```

- **Browser JavaScript (`dust_dds_wasm`)**:
  ```javascript
  const valueType = DynamicTypeBuilderFactory.getPrimitiveType(TypeKind.Float64);
  const valueMember = new MemberDescriptor("value", valueType, 0, false, false);

  const sineBuilder = DynamicTypeBuilderFactory.createStructureType("SineWave");
  sineBuilder.addMember(valueMember);
  const sineDynamicType = sineBuilder.build();

  // History KeepLast(10)
  const sineTopicQos = new TopicQos();
  sineTopicQos.setHistory(new HistoryQosPolicy(HistoryQosPolicyKind.KeepLast, 10));

  const readerQos = new DataReaderQos();
  readerQos.setHistory(new HistoryQosPolicy(HistoryQosPolicyKind.KeepLast, 10));

  const sineListener = {
    onDataAvailable: async (reader) => {
      const samples = await reader.read(10);
      const values = samples
        .filter((s) => s.getSampleInfo().getValidData())
        .map((s) => s.getData().value);
      drawSinePlot(values);
    },
  };
  ```

---

## How to Run

### 1. Build WebAssembly Package (Optional / Already Pre-built)

If you modify `bindings/wasm`, recompile the WASM package:

```bash
cargo build --package dust_dds_wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir bindings/wasm/demo/browser_publisher/pkg target/wasm32-unknown-unknown/debug/dust_dds_wasm.wasm
```

### 2. Run the Native Application

From the root of the workspace:

```bash
cargo run -p native_reader
```

This starts:
- The WebRTC/WebSocket transport server on `ws://127.0.0.1:9001`.
- A built-in HTTP server on `http://127.0.0.1:8080` serving the browser application.
- A DDS `DataReader` listening for `TextMessage` samples.
- A periodic task publishing `SineWave` values every 0.1 seconds (100ms).

### 3. Open the Browser Application

> [!NOTE]
> **Loopback WebRTC in Chromium**: Per RFC 8445 and Chromium security policies, WebRTC ignores `127.0.0.1` (loopback) interfaces by default when connecting to peers on the same local machine. If running Chromium/Chrome on the same machine as `native_reader`, launch Chrome with the loopback flag:
> ```bash
> google-chrome --allow-loopback-in-peer-connection http://127.0.0.1:8080
> # or on chromium:
> chromium --allow-loopback-in-peer-connection http://127.0.0.1:8080
> ```

Open your web browser and navigate to:

```
http://127.0.0.1:8080
```

1. The WebSocket connection (`ws://127.0.0.1:9001`) is used **exclusively for discovery / signaling** (peer registration, SDP offer/answer exchange, ICE candidates). Zero RTPS user data or SEDP discovery traffic flows over the WebSocket.
2. The WebRTC DataChannel (`"dust_rtps"`) opens directly between the browser and native reader.
3. Watch the **Sine Wave Monitor** automatically plot the last 10 received samples on the canvas in real time as they stream directly over the WebRTC DataChannel.
4. Type any message in the input box (or click one of the quick suggestions) and click **Publish**.
5. Observe the native terminal print the received `TextMessage` in real time!
