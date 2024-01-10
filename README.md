# LibCSP-rs

A safe LibCSP wrapper for Rust.

Currently, this wrapper is planned to support only the features necessary for the Toliman mission.

## Compiling

For the project to compile, the relevant headers need to be installed in the system, as well as the dynamic library compiled with the features that are enabled in this crate.

For reference, use `shell.nix`.

In the future, I may add static linking to avoid the need for having the dynamic library installed in the system.

## Compatibility

This crate uses LibCSP version `v1.6`. As of writing, `v1.6` is 4 years old, while the libcsp repository is still active working on the unfinished `v2.0`.
