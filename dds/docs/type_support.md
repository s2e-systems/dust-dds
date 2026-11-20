# Defining DDS Types in Rust (`#[derive(DdsType)]`) and IDL

This document provides a comprehensive reference for defining Data Distribution Service (DDS) types in Rust using Dust DDS and generating types from OMG Interface Definition Language (IDL) files.

---

## The `#[derive(DdsType)]` Macro

The `dust_dds` crate provides the `DdsType` derive macro (via `dust_dds_derive`) to automatically implement the traits required for DDS topic communication, serialization, deserialization, key extraction, and dynamic TypeObject generation.

The `DdsType` derive macro can be applied to `struct`s, tuples, and `enum`s.

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
struct SensorData {
    #[dust_dds(key)]
    sensor_id: u32,
    value: f64,
}
```

---

## Container Attributes

Container attributes apply to the entire `struct` or `enum` definition and are specified using `#[dust_dds(...)]` on the container:

* **`name = "<string>"`**:
  Customizes the registered DDS type name. Defaults to the Rust struct or enum identifier.
* **`extensibility = "final" | "appendable" | "mutable"`**:
  Specifies the type extensibility defined in the OMG DDS XTypes specification:
  - `"final"` (default): Fields cannot be added or modified in future versions without breaking compatibility.
  - `"appendable"`: New fields can be appended at the end of the type in future revisions.
  - `"mutable"`: Fields are identified by member IDs and can be reordered, omitted, or added arbitrarily.
* **`nested`**:
  A boolean flag marking the type as nested. Nested types cannot be used as standalone top-level topics but can be embedded as members in other DDS types.
* **`base_type = <Type>`**:
  Specifies the base struct for single inheritance. When specified, the struct must declare a field named `parent` of type `<Type>`.

---

## Struct Field Attributes

Field attributes apply to individual fields of a struct or tuple, specified using `#[dust_dds(...)]` on the field:

* **`key`**:
  Marks the field as part of the unique DDS instance key. Multiple fields can be marked as keys to form a composite key.
* **`id = <integer>`**:
  Explicitly assigns a member ID. This is particularly useful for `"mutable"` extensibility to guarantee member ID stability across languages and revisions.
* **`optional`**:
  Marks the field as optional (typically wrapped in `Option<T>`).
* **`default_value = <expression>`**:
  Specifies a default value expression to use if the field is missing during deserialization.
* **`non_serialized`**:
  Marks a field to be skipped during wire serialization and deserialization. It is initialized locally using `Default::default()`.
* **`external`**:
  Marks the field as external, indicating pointer/box/reference semantic representation in the DDS type system.
* **`hashid`**:
  Instructs the macro to automatically assign a member ID computed from the MD5 hash of the field's name (first 4 bytes as a little-endian integer).
* **`try_construct = "discard" | "use_default" | "trim"`**:
  Specifies the try-construct behavior for deserialization errors (defined in OMG DDS XTypes). Defaults to `"discard"`.

---

## Struct Inheritance (`base_type`)

DDS XTypes supports single inheritance for structures (corresponding to `struct Child : Parent` in OMG IDL). Because Rust does not support struct inheritance natively, Dust DDS follows the OMG DDS-XTypes specification mapping for languages without native inheritance:

1. Annotate the derived struct with `#[dust_dds(base_type = <BaseType>)]`.
2. Declare a field named `parent` of type `<BaseType>` in the derived struct.

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
struct Vehicle {
    #[dust_dds(key)]
    vin: String,
    speed: f32,
}

#[derive(DdsType)]
#[dust_dds(base_type = Vehicle)]
struct Car {
    parent: Vehicle,
    number_of_doors: u8,
}
```

When `base_type` is configured:
* **Field access**: Fields of the base type are accessed through the `parent` field (e.g., `car.parent.vin`).
* **Serialization**: Base struct members are serialized and deserialized first, followed by the derived struct's own members.
* **Member IDs**: Member IDs for the derived struct's fields automatically continue after the base type's member IDs (for `final` and `appendable` structs, or auto-assigned in `mutable` structs).
* **TypeObject**: The generated `TypeObject` links the derived type to the base type via `header.base_type`, preserving the inheritance hierarchy across DDS discovery and type consistency checks.

---

## Enum Attributes: Enumerations and Unions

An enum can represent either a standard **Enumerated Type** or an **XTypes Union**.

### 1. Enumerated Types (Unit Variants Only)

If an enum contains only unit variants (variants without fields), it maps to a DDS enumerated type:

* **`bit_bound = "8" | "16" | "32"`**:
  Specifies the size of the underlying integer representation in bits. Defaults to `"32"`.
* Custom integer discriminants (e.g., `Red = 1`) are mapped directly to their corresponding DDS values.

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
#[dust_dds(bit_bound = "16")]
enum TrafficLight {
    Red = 1,
    Yellow = 2,
    Green = 3,
}
```

### 2. XTypes Unions (Variants with Fields)

If an enum contains at least one variant with fields, it maps to an OMG DDS XTypes Union:

* **`switch(<Type>)` or `switch(key, <Type>)`**:
  **Required.** Specifies the type of the discriminator. If `key` is present, the discriminator is treated as part of the instance key.
* **`case = <expression>`** (on variants):
  Specifies the discriminator value(s) for which this variant is selected. If omitted, defaults to the 0-indexed index of the variant.
* **`default`** (on a variant):
  Marks the variant as the default case when no explicit case matches.

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
#[dust_dds(switch(i32))]
enum Shape {
    #[dust_dds(case = 1)]
    Circle(f64),
    #[dust_dds(case = 2)]
    Square { side: f64 },
    #[dust_dds(default)]
    Unknown,
}
```

---

## Comprehensive Examples

### Appendable and Nested Struct

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
#[dust_dds(name = "CustomPoint", extensibility = "appendable", nested)]
struct Point {
    x: f64,
    y: f64,
}
```

### Mutable Struct with Explicit Member IDs and Defaults

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
struct Profile {
    #[dust_dds(id = 1, key)]
    id: u32,
    #[dust_dds(id = 2)]
    username: String,
    #[dust_dds(id = 10, optional)]
    email: Option<String>,
    #[dust_dds(id = 11, default_value = 18)]
    age: u32,
    #[dust_dds(non_serialized)]
    session_count: u32,
}
```

---

## IDL Type Generation (`dust_dds_gen`)

When interoperating with existing systems, other DDS vendors (Fast DDS, Cyclone DDS, RTI Connext), or multi-language projects (C, C++, Python), DDS types can be defined in standard OMG IDL v4 files and compiled into Rust code using the [`dust_dds_gen`](https://crates.io/crates/dust_dds_gen) tool.

### Example IDL Definition

```idl
struct HelloWorldType {
  @key
  uint8 id;
  string msg;
};
```

### Inheritance in IDL

Inheritance in IDL is supported and automatically translated to Rust using `base_type` and the `parent` field:

```idl
struct Parent {
  uint8 a;
};

struct Child : Parent {
  string name;
};
```

### Using `dust_dds_gen`

Install `dust_dds_gen` via Cargo:

```bash
cargo install dust_dds_gen
dust_dds_gen -o src/generated messages.idl
```

Or integrate it into a `build.rs` build script to generate Rust structs during compilation.
