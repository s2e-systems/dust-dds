# Derive macro for `DdsType`

This package provides a derive macro for `DdsType` to support [dust-dds](https://github.com/s2e-systems/dust-dds).

`DdsType` can only be derived for `struct`s, tuples and `enum`s. For `struct`s and tuples, the attribute `#[dust_dds(key)]` can be specified either on the whole type or on a subset of fields.

## Example

A typical user DDS type will look like this:

```rust
use dust_dds::topic_definition::type_support::{DdsType}

#[derive(DdsType)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

```
