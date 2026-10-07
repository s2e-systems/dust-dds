# DDS Security: Submessage and RTPS Message Encryption Architecture & Implementation Plan

This document outlines the architectural design and step-by-step implementation plan for adding **Submessage Encryption** and **RTPS Message Encryption** to `dust-dds`, in conformance with the **OMG DDS Security Specification v1.2** (`formal/25-03-06`).

---

## 1. Scope & Security Levels in DDS

The DDS Security specification defines three distinct levels of message and data protection:

| Protection Level | Responsible SPI Method | Governing Configuration | Generated Wire Format |
|---|---|---|---|
| **Payload Encryption** | `CryptoTransform::encode_serialized_payload` / `decode_serialized_payload` | `EndpointSecurityConfig::is_payload_protected` | Replaces `SerializedPayload` inside `DATA`/`DATAFRAG` with `CryptoHeader` + `CryptoContent` + `CryptoFooter` |
| **Submessage Encryption & Auth** | `CryptoTransform::encode_datawriter_submessage` / `encode_datareader_submessage` | `EndpointSecurityConfig::is_submessage_protected` | Wraps submessages (`DATA`, `GAP`, `HEARTBEAT`, `ACKNACK`) into `SEC_PREFIX` + `SEC_BODY` (or plain) + `SEC_POSTFIX` |
| **RTPS Message Encryption & Auth** | `CryptoTransform::encode_rtps_message` / `decode_rtps_message` | `ParticipantSecurityConfig::is_rtps_axk_protected` and `is_rtps_psk_protected` | Wraps full RTPS message body into `SRTPS_PREFIX` + `SEC_BODY` (or plain) + `SRTPS_POSTFIX` |

---

## 2. Storage and Placement of `EndpointSecurityConfig`

### Architectural Decision

> **`EndpointSecurityConfig` shall be stored on `DataWriterEntity` (and symmetrically on `DataReaderEntity`), NOT on `RtpsStatefulWriter`.**

### Technical Rationale

1. **Architectural Layering (DCPS vs. RTPS Separation)**:
   - `EndpointSecurityConfig` (§9.4.2.7) is a DDS Access Control SPI concept derived from `TopicSecurityConfig` (§9.4.2.6). It holds policy decisions (`is_read_protected`, `is_write_protected`, `is_discovery_protected`), plugin properties (`ac_endpoint_properties`), and cipher suites (`algorithm_info`).
   - `RtpsStatefulWriter` is a pure transport-agnostic RTPS protocol state machine managing cache changes, sequence numbers, and reader proxies according to DDSI-RTPS 2.5. Injecting high-level DDS Access Control configuration directly into `RtpsStatefulWriter` introduces tight coupling and violates separation of concerns.

2. **Entity Symmetry (`DataWriter` and `DataReader`)**:
   - Both `DataWriter`s and `DataReader`s have associated `EndpointSecurityConfig` instances (§9.4.2.9.24 and §9.4.2.9.25).
   - DataReaders send protocol submessages (`AckNack`, `NackFrag`) that can be encrypted/signed via `encode_datareader_submessage`.
   - Storing `EndpointSecurityConfig` in `DataWriterEntity<T>` and `DataReaderEntity<T>` maintains symmetry across the DCPS layer without coupling to specific RTPS endpoint implementations.

3. **Polymorphic Writers (`StatefulWriter` vs. `StatelessWriter`)**:
   - In `dust-dds`, `DataWriterEntity<T: RtpsWriter>` abstracts both `RtpsStatefulWriter` and `RtpsStatelessWriter`.
   - Builtin secure discovery endpoints or stateless endpoints can have submessage protection (Table 21). Storing security configuration at the `DataWriterEntity` layer allows stateless and stateful writers to share uniform security metadata handling.

4. **Configuration vs. Cryptographic Handles**:
   - `EndpointSecurityConfig` is used primarily during:
     - **Entity Creation / Registration**: Passed to `CryptoKeyFactory::register_local_datawriter` to establish local key material and obtain the opaque `DatawriterCryptoHandle`.
     - **Discovery Announcement**: Used to construct `PublicationBuiltinTopicDataSecure` (`PID_ENDPOINT_SECURITY_PROTECTION_INFO` and `PID_ENDPOINT_SECURITY_SYMMETRIC_CIPHER_ALGORITHM_INFO`).
     - **Matching & Key Exchange**: Triggers key distribution when `is_submessage_protected || is_payload_protected`.
     - **Entity Destruction**: Returned to the plugin via `AccessControl::return_datawriter_security_config`.
   - At runtime, cryptographic transformation functions (`encode_datawriter_submessage`, `encode_serialized_payload`) require **opaque crypto handles** (`DatawriterCryptoHandle` and destination `DatareaderCryptoHandle`s), not the raw `EndpointSecurityConfig`.

5. **Participant Scope for RTPS Message Encryption**:
   - RTPS message-level encryption (`encode_rtps_message`) is participant-scoped and controlled by `ParticipantSecurityConfig` (`is_rtps_axk_protected` / `is_rtps_psk_protected`), using `ParticipantCryptoHandle`.
   - An individual `RtpsStatefulWriter` cannot govern RTPS message encryption because an RTPS message frequently bundles submessages from multiple entities or participant-level endpoints.

### Proposed Structure

Define an `EndpointSecurity` container stored inside `DataWriterEntity` and `DataReaderEntity`:

```rust
pub struct EndpointSecurity<CryptoHandle> {
    pub config: EndpointSecurityConfig,
    pub crypto_handle: CryptoHandle,
}

pub struct DataWriterEntity<T> {
    pub instance_handle: InstanceHandle,
    pub transport_writer: T,
    pub topic_name: Arc<str>,
    pub enabled: bool,
    pub last_change_sequence_number: i64,
    pub qos: DataWriterQos,
    pub registered_instance_info: Vec<RegisteredInstanceInfo>,
    pub key_holder_type: KeyHolderType,
    /// Security configuration and local cryptographic handle
    pub security: Option<EndpointSecurity<DatawriterCryptoHandle>>,
}
```

---

## 3. Wire Protocol Specifications (`rtps_messages`)

DDS Security v1.2 (§7.4.6 & §7.4.7) defines 5 secure submessages and 3 secure submessage elements.

### 3.1 Submessage Identifiers

| Submessage Name | Wire Constant | Value | Purpose |
|---|---|---|---|
| `SecureBodySubMsg` | `SEC_BODY` | `0x30` | Wraps encrypted submessage or RTPS message contents (`CryptoContent`) |
| `SecurePrefixSubMsg` | `SEC_PREFIX` | `0x31` | Precedes a secured submessage, carrying `CryptoHeader` |
| `SecurePostfixSubMsg` | `SEC_POSTFIX` | `0x32` | Follows a secured submessage, carrying `CryptoFooter` (MAC / signatures) |
| `SecureRTPSPrefixSubMsg` | `SRTPS_PREFIX` | `0x33` | Precedes a secured RTPS message, carrying `CryptoHeader` and flags |
| `SecureRTPSPostfixSubMsg` | `SRTPS_POSTFIX` | `0x34` | Follows a secured RTPS message, carrying `CryptoFooter` |

### 3.2 Secure Submessage Elements (§7.4.7)

1. **`CryptoTransformIdentifier`**:
   - `transformation_kind: CryptoTransformKind` (algorithm ID + key revision).
   - `transformation_key_id: CryptoTransformKeyId` (4-byte key ID).
2. **`CryptoHeader`**:
   - Extends `CryptoTransformIdentifier`.
   - `plugin_crypto_header_extra`: for AES-GCM-GMAC (§10.5.2.3), includes `session_id: [u8; 4]` and `initialization_vector_suffix: [u8; 8]`. Serialized Big Endian.
3. **`CryptoContent`**:
   - Length prefix (`u32` Big Endian) + sequence of cipher bytes (multiple of 16 bytes for AES).
4. **`CryptoFooter`**:
   - For AES-GCM-GMAC (§10.5.2.5):
     - `common_mac: [u8; 16]`
     - `receiver_specific_macs: Vec<ReceiverSpecificMAC>` where each entry is `receiver_mac_key_id: [u8; 4]` + `receiver_mac: [u8; 16]`.

### 3.3 Submessage Wire Structures

- **Submessage Protection (§10.5.3.3.4.5)**:
  - If **Encryption**:
    `SEC_PREFIX` + `SEC_BODY(crypto_content = Encrypt(SubMessage))` + `SEC_POSTFIX(CryptoFooter)`
  - If **Authentication Only**:
    `SEC_PREFIX` + `SubMessage (plaintext)` + `SEC_POSTFIX(CryptoFooter)`
- **RTPS Message Protection (§10.5.3.3.4.6)**:
  - If **Encryption with AAD**:
    `RTPSHeader` + `SRTPS_PREFIX(A=1)` + `SEC_BODY(crypto_content = Encrypt(SubMessages))` + `SRTPS_POSTFIX(CryptoFooter with RTPSHeader as AAD)`
  - If **Authentication Only with AAD**:
    `RTPSHeader` + `SRTPS_PREFIX(A=1)` + `SubMessages (plaintext)` + `SRTPS_POSTFIX(CryptoFooter with RTPSHeader as AAD)`

---

## 4. Architectural Changes Across `dust-dds`

### 4.1 `dds/src/rtps_messages`
- Implement types and parsers/writers for:
  - `CryptoTransformIdentifier`, `CryptoHeader`, `CryptoContent`, `CryptoFooter`, `ReceiverSpecificMAC`.
  - `SecurePrefixSubmessage`, `SecureBodySubmessage`, `SecurePostfixSubmessage`.
  - `SecureRTPSPrefixSubmessage`, `SecureRTPSPostfixSubmessage`.
- Update `SubmessageKind` enum and `RtpsSubmessageReadKind` union to recognize kinds `0x30`..`0x34`.

### 4.2 `dds/src/rtps` (Proxy Crypto Handles & Decoupled Message Construction)
- **`RtpsReaderProxy`**:
  - Add optional `remote_datareader_crypto: DatareaderCryptoHandle`.
- **`RtpsWriterProxy`**:
  - Add optional `remote_datawriter_crypto: DatawriterCryptoHandle`.
- **Decouple Submessage Creation from Direct Transmission**:
  - Currently, `RtpsStatefulWriter::write_message` directly serializes an entire `RtpsMessageWrite` into `message_writer.write_buffer_mut()` and calls `write_message(...)`.
  - Introduce an intermediate stage:
    - Writers generate submessages (`Submessage` trait objects or serialized submessage byte slices).
    - If `is_submessage_protected` is active, apply `CryptoTransform::encode_datawriter_submessage`.
    - Package into `RtpsMessageWrite`.
    - If participant RTPS message protection is enabled, apply `CryptoTransform::encode_rtps_message`.
    - Transmit over transport.

### 4.3 `dds/src/dcps` (Entity Lifecycle & Security Context)
- **DataWriter Creation** (`DcpsDomainParticipant::create_data_writer` in `publisher_methods.rs`):
  - When security is active:
    1. Call `AccessControl::get_datawriter_security_config`.
    2. Call `CryptoKeyFactory::register_local_datawriter` to obtain `DatawriterCryptoHandle`.
    3. Store `EndpointSecurity` inside `DataWriterEntity`.
    4. Set up `PID_ENDPOINT_SECURITY_PROTECTION_INFO` and `PID_ENDPOINT_SECURITY_SYMMETRIC_CIPHER_ALGORITHM_INFO` in `DiscoveredWriterData`.
- **DataReader Creation** (`DcpsDomainParticipant::create_data_reader` in `subscriber_methods.rs`):
  - Perform symmetric initialization for `DataReaderEntity`.
- **Endpoint Matching**:
  - Upon matching a discovered remote reader/writer:
    1. Call `CryptoKeyFactory::register_matched_remote_datareader` or `register_matched_remote_datawriter`.
    2. Store the returned remote crypto handle in the corresponding reader/writer proxy.

### 4.4 `dds/src/dcps` (Inbound Message Decoding Pipeline)
Refactor `DcpsDomainParticipant::handle_data` (`communication_methods.rs`):
1. **RTPS Message Decryption**:
   - Check if the incoming packet starts with `SRTPS_PREFIX` (0x33).
   - If present, resolve the sender's `ParticipantCryptoHandle` and invoke `CryptoTransform::decode_rtps_message` to obtain the plaintext RTPS message.
2. **Submessage Decryption**:
   - When iterating submessages in `MessageReceiver`:
     - If `SEC_PREFIX` (0x31) is encountered:
       - Call `CryptoTransform::preprocess_secure_submsg` to determine whether it is a writer or reader submessage and retrieve the crypto handles.
       - Call `decode_datawriter_submessage` or `decode_datareader_submessage`.
       - Yield the decrypted submessage into the standard dispatch flow (`handle_data_submessage`, `handle_heartbeat_submessage`, `handle_gap_submessage`, etc.).
3. **Payload Decryption**:
   - If `DATA` or `DATAFRAG` contains `CryptoContent` instead of `SerializedPayload`:
     - Invoke `CryptoTransform::decode_serialized_payload` to retrieve original sample bytes.

### 4.5 Key Exchange Infrastructure (§7.5.4)
- Instantiate builtin `DCPSParticipantVolatileMessageSecure` endpoints:
  - `BuiltinParticipantVolatileMessageSecureWriter` (Reliable StatefulWriter, Volatile).
  - `BuiltinParticipantVolatileMessageSecureReader` (Reliable StatefulReader, Volatile).
- Use these endpoints to send/receive:
  - `ParticipantCryptoTokenSeq` (`GMCLASSID_SECURITY_PARTICIPANT_CRYPTO_TOKENS`).
  - `DatawriterCryptoTokenSeq` (`GMCLASSID_SECURITY_DATAWRITER_CRYPTO_TOKENS`).
  - `DatareaderCryptoTokenSeq` (`GMCLASSID_SECURITY_DATAREADER_CRYPTO_TOKENS`).

---

## 5. Phased Implementation Plan

### Phase 1: Wire Format & Cryptographic Submessages
- [ ] Add submessage constants (`0x30`..`0x34`) to `rtps_messages/types.rs`.
- [ ] Implement `CryptoHeader`, `CryptoContent`, and `CryptoFooter` in `rtps_messages/submessage_elements.rs`.
- [ ] Implement `SecurePrefixSubmessage`, `SecureBodySubmessage`, `SecurePostfixSubmessage`, `SecureRTPSPrefixSubmessage`, and `SecureRTPSPostfixSubmessage` in `rtps_messages/submessages/`.
- [ ] Update `SubmessageKind` and `RtpsSubmessageReadKind` in `rtps_messages/overall_structure.rs`.
- [ ] Write unit tests for serialization and parsing of all secure submessages.

### Phase 2: Entity Lifecycle & Security Context Binding
- [ ] Define `EndpointSecurity<CryptoHandle>` in `dcps_domain_participant`.
- [ ] Add `security: Option<EndpointSecurity<DatawriterCryptoHandle>>` to `DataWriterEntity`.
- [ ] Add `security: Option<EndpointSecurity<DatareaderCryptoHandle>>` to `DataReaderEntity`.
- [ ] Update `create_data_writer` and `create_data_reader` to query access control and register local crypto handles.
- [ ] Update entity destruction to call `return_datawriter_security_config` / `return_datareader_security_config` and `unregister_datawriter` / `unregister_datareader`.
- [ ] Propagate `EndpointSecurityProtectionInfo` in discovery data.

### Phase 3: Outgoing Message Transformation Pipeline
- [ ] Add remote crypto handles (`DatareaderCryptoHandle` / `DatawriterCryptoHandle`) to `RtpsReaderProxy` and `RtpsWriterProxy`.
- [ ] Refactor `RtpsStatefulWriter` and `reader_proxy` to decouple submessage generation from direct socket writes.
- [ ] Implement Submessage Crypto Transform hook:
  - If `is_submessage_protected`, invoke `encode_datawriter_submessage` / `encode_datareader_submessage`.
- [ ] Implement RTPS Message Crypto Transform hook:
  - If `is_rtps_axk_protected` or `is_rtps_psk_protected`, invoke `encode_rtps_message`.
- [ ] Write unit tests verifying transformation produces expected `SEC_PREFIX`/`SEC_BODY`/`SEC_POSTFIX` and `SRTPS_PREFIX`/`SRTPS_POSTFIX` packets.

### Phase 4: Inbound Message Decoding Pipeline
- [ ] Update `DcpsDomainParticipant::handle_data` to check for `SRTPS_PREFIX` and invoke `decode_rtps_message`.
- [ ] Update `MessageReceiver` to detect `SEC_PREFIX`, call `preprocess_secure_submsg`, and dispatch through `decode_datawriter_submessage` / `decode_datareader_submessage`.
- [ ] Integrate with `decode_serialized_payload` when `CryptoContent` is present in `DataSubmessage`.
- [ ] Add unit and roundtrip tests verifying that encoded messages are decoded back to the original submessages.

### Phase 5: Key Distribution via Volatile Secure Builtin Endpoints
- [ ] Create `BuiltinParticipantVolatileMessageSecureWriter` and `BuiltinParticipantVolatileMessageSecureReader` in `BuiltinPublisher` / `BuiltinSubscriber` (§7.5.4).
- [ ] Implement derivation of key exchange keys (`KxKey`, `KxMacKey`) from `SharedSecret` (§10.5.2.1.2).
- [ ] Implement exchange of `ParticipantCryptoTokenSeq` and `DatawriterCryptoTokenSeq` / `DatareaderCryptoTokenSeq` upon participant and endpoint matching (§9.8.10).
- [ ] Wire received crypto tokens to `CryptoKeyExchange::set_remote_*_crypto_tokens`.

### Phase 6: Integration & Verification
- [ ] Add integration tests for communication between two secured `DomainParticipant`s with submessage protection enabled (`is_submessage_protected = true`).
- [ ] Add integration tests for communication with RTPS message protection enabled (`is_rtps_axk_protected = true` and `is_rtps_psk_protected = true`).
- [ ] Verify tamper resistance and replay attack rejection via invalid MAC injection tests.
- [ ] Verify interoperability with standard compliance test suites.
