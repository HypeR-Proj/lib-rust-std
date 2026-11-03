<!--
SPDX-FileCopyrightText: 2026 roolrz
SPDX-License-Identifier: Apache-2.0
-->

# lib-rust-std

Rust standard-library port and shared runtime owner for Native applications.

pal/ adapts the pinned Rust std source to Native services; shim/ owns its C bridge, and
tools/ installs and checks the overlay. The dylib target reexports one std instance for
same-build Rust shared libraries.

See the [implementation entry](src/lib.rs) and [component declaration](component.json)
for the public interfaces, build dependencies and delivered artifacts.

## Boundaries

Unsupported std facilities retain their upstream error or panic behavior; this port does
not imply POSIX or network-stack completeness. The SDK toolchain, PAL, C shim and ABI
records must be built together. component.json declares the delivered library and ABI
layout checks.

## Build and test

Use this checkout inside the [repo-managed workspace](https://github.com/HypeR-Proj/HypeR).
From this repository directory:

```sh
make
make test
```

An absolute `OUT=/path/to/out` selects a shared output directory. The module
entry delegates to build/system, which prepares the installed SDK and declared
dependencies. Native binaries require HypeR; host module checks cover the tests
selected by component.json. A standalone clone does not contain the workspace’s
other source repositories or Native SDK.


The prepared PAL includes the ABI candidate's generated Rust bindings. Native
filesystem kinds and flags come from that candidate; compile-time checks compare
every bridge record field, size and alignment with its schema-owned counterpart.
The std open options are a separate SDK-private domain and are checked against
the C header by `tools/check-abi.py`. Preparing an overlay requires the ABI source
as the fourth argument to `tools/prepare-rust-std.py`.
