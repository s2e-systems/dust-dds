# Derive macro for `DdsHasKey`

This package provides a derive macro for `DdsHasKey` to support [dust-dds](https://github.com/s2e-systems/dust-dds).

`DdsHasKey` can only be derived for `struct`s, tuples and `enum`s. For `struct`s and tuples, the attribute `#[key]` can be specified either on the whole type or on a subset of fields. For `enum`s, the `#[key]` attribute can only be specified on the whole type.

## Example

A typical user DDS type will look like this:

```rust
use dust_dds::topic_definition::type_support::{DdsHasKey}
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, DdsHasKey)]
struct HelloWorldType {
    #[key]
    id: u8,
    msg: String,
}

```
