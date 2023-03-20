# Windsor

## Directory Overview

#### windsor-boot

First-stage bootloader, responsible for unpacking a zstd-compressed kernel,
setting up its initial environment, and jumping to it.

#### windsor-kernel

The kernel itself.

#### build-tool

A build staging tool, as an alternative to Makefiles. Responsible for building
individual components in the correct sequence and post-processing the generated
ELF files.

#### build-macros

Includes a proc-macro for including the compressed kernel artifact inside windsor-boot,
complete with information required to load it correctly. This has a dependency on build-tool.

## Workspace Note

Due to limitations with cargo workspaces and cargo-std-aware, the four projects above cannot
be contained within the same workspace without some drawback. As such, only build-tool and build-macros share a workspace.

windsor-boot and windsor-kernel are standalone projects, excluded by the workspace. If you are using
rust-analyzer while editing code, you will have to open your editor within the sub-project directory
(i.e. within `./windsor-boot` or `./windsor-kernel`) instead of the repo root (or otherwise configure
rust-analyzer to use the sub-project directory over the workspace).

## Dependencies

```sh
$ rustup toolchain install nightly --component rust-src
```

## Building

In the repo root, simply run the `build-tool`:

```sh
$ cargo run
```

To clean, run:

```sh
$ cargo run clean
```
