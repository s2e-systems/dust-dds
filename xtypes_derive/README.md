# Derive macro for `XTypes`

This package provides a derive macro for `XTypes` to support [dust-dds](https://github.com/s2e-systems/dust-dds).

`XTypes` can only be derived for `struct`s, tuples and `enum`s. For `struct`s and tuples, the attribute `#[dust_dds(key)]` can be specified either on the whole type or on a subset of fields.

## Example

A typical user type will look like this:

```rust
use dust_xtypes::serialize::XTypesSerialize;

#[derive(XTypesSerialize)]
struct HelloWorldType {
    id: u8,
    msg: String,
}

```
