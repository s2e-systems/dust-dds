# Derive macro for `DdsType`

This package provides a derive macro for `DdsType` to support [dust-dds](https://github.com/s2e-systems/dust-dds).

`DdsType` can only be derived for `struct`s, tuples and `enum`s. For `struct`s and tuples, the attribute `#[dust_dds(key)]` can be specified either on the whole type or on a subset of fields.

## Example

A typical user DDS type will look like this:

```rust
use dust_dds::infrastructure::type_support::{DdsType}

#[derive(DdsType)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}
```

## Mutable Structs with Optional Field IDs

For mutable extensibility types, you can now use optional field IDs. If not provided, IDs will auto-increment starting from 0:

```rust
#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
struct MutableType {
    #[dust_dds(key)]
    id: u8,           // Auto-assigned ID: 0
    name: String,     // Auto-assigned ID: 1
    age: u32,         // Auto-assigned ID: 2
}
```

You can also mix explicit and auto-generated IDs:

```rust
#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
struct MixedIdType {
    #[dust_dds(id = 10, key)]
    id: u8,           // Explicit ID: 10
    name: String,     // Auto-assigned ID: 0
    age: u32,         // Auto-assigned ID: 1
}
```

