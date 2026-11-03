<!--
SPDX-FileCopyrightText: 2026 roolrz
SPDX-License-Identifier: Apache-2.0
-->

# lib-rust-std

A HypeR component. See [HypeR](https://github.com/HypeR-Proj/HypeR) for
workspace setup and the installed SDK build boundary.

## Workspace tests

Use the [HypeR workspace](https://github.com/HypeR-Proj/HypeR) to resolve dependencies:

```sh
python3 project.py test lib-rust-std
```

Paths in the documentation are relative to the repo-managed workspace root.
Edit this repository and prepare the workspace again before building.

The prepared PAL includes the ABI candidate's generated Rust bindings. Native
filesystem kinds and flags come from that candidate; compile-time checks compare
every bridge record field, size and alignment with its schema-owned counterpart.
The std open options are a separate SDK-private domain and are checked against
the C header by `tools/check-abi.py`. Preparing an overlay requires the ABI source
as the fourth argument to `tools/prepare-rust-std.py`.
