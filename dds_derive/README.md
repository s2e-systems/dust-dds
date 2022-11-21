# Derive macro for `DdsType`

This package provides derive macros for `DdsType` and `DdsSerde` to support [dust-dds](https://github.com/s2e-systems/dust-dds).

`DdsType` can only be derived for `struct`s, tuples and `enum`s. For `struct`s and tuples, the attribute `#[key]` can be specified either on the whole type or on a subset of fields. For `enum`s, the `#[key]` attribute can only be specified on the whole type.

## Example

A typical user DDS type will look like this:

```rust
use dust_dds::topic_definition::type_support::{DdsSerde, DdsType}
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, DdsType, DdsSerde)]
struct HelloWorldType {
    #[key]
    id: u8,
    msg: String,
}

```
