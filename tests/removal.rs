// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

// Compile the actual platform retry policy against host std for deterministic
// delayed-retirement, replacement and permanent-error tests.
pub use std::{cell, io, thread, time};

#[path = "../pal/overlay/std/src/sys/fs/hyper/removal.rs"]
mod removal;
