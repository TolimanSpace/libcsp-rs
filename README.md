# LibCSP-rs

A safe LibCSP wrapper for Rust.

Currently, this wrapper is planned to support only the features necessary for the Toliman mission.

## Compiling

For the project to compile, the relevant headers need to be installed in the system, as well as the dynamic library compiled with the features that are enabled in this crate.

For reference, use `shell.nix`.

In the future, I may add static linking to avoid the need for having the dynamic library installed in the system.

## Testing

### Standard Tests

To run the standard tests, use the following command inside the `nix-shell`:

```bash
cargo test
```

### Interoperability Tests

The interoperability tests ensure that the Rust wrapper works correctly with the C implementation. These tests are marked as `#[ignore]` by default and must be run inside the `nix-shell`:

```bash
cargo test -p libcsp --test interop -- --ignored --nocapture
```

## Compatibility

This crate uses LibCSP version `v1.6`. As of writing, `v1.6` is 4 years old, while the libcsp repository is still active working on the unfinished `v2.0`.

## Caveats

There are a lot of options in LibCSP that are very unclear, I ended up choosing defaults for many things such as connection bitflags and some timeouts. If you need to modify those for any reason, feel free to PR into the library with a better solution. Just make sure you make it configurable with standard Rust practices, e.g. the `bitflags` crate.
