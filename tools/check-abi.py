#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 roolrz
# SPDX-License-Identifier: Apache-2.0

"""Compile the PAL and C shim against the same Native record/flag contract."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--abi', type=Path, required=True)
parser.add_argument('--system', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
owner = Path(__file__).resolve().parents[1]
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
shutil.copyfile(owner / 'pal/overlay/std/src/sys/pal/hyper/ffi.rs', output / 'ffi.rs')
shutil.copyfile(args.abi / 'src/generated.rs', output / 'native_abi.rs')
header = (owner / 'shim/include/hyper/std.h').read_text()
options = re.findall(r'^#define HYPER_STD_OPEN_(\w+) UINT32_C\((\d+)\)$', header, re.M)
if len(options) != 6:
    raise SystemExit('expected the six named std open options')
checks = '\n'.join(f'const _: () = assert!(ffi::OPEN_{name} == {value});' for name, value in options)
(output / 'check.rs').write_text('#![allow(dead_code)]\nmod ffi;\n' + checks + '\n')
subprocess.run(['rustc', '--edition=2024', '--crate-type=lib', '--emit=metadata',
                str(output / 'check.rs'), '-o', str(output / 'check.rmeta')], check=True)
subprocess.run([os.environ.get('CLANG', 'clang'), '-std=c11', '-fsyntax-only',
                '-Werror', '-I', str(owner / 'shim/include'),
                '-I', str(args.system.resolve() / 'include'),
                '-I', str(args.abi.resolve() / 'include'),
                str(owner / 'shim/src/std-fs.c'),
                str(owner / 'shim/src/std.c'),
                str(owner / 'shim/src/std-process.c')], check=True)

# Exercise the actual PAL fill adapter without a Native process or entropy
# dependency. Errors must panic, never replace missing bytes with a weak seed.
random_source = owner / 'pal/overlay/std/src/sys/random/hyper.rs'
(output / 'random-check.rs').write_text('''
use std::sync::atomic::{AtomicBool, Ordering};
static FAIL: AtomicBool = AtomicBool::new(false);
mod sys { pub mod pal { pub mod ffi {
    pub unsafe fn __hyper_std_random_fill(buffer: *mut u8, length: usize) -> i64 {
        if crate::FAIL.load(crate::Ordering::Relaxed) { return -23; }
        unsafe { std::ptr::write_bytes(buffer, 0xa5, length); }
        0
    }
}}}
''' + f'#[path = {json.dumps(str(random_source))}]\nmod random;\n' + '''
#[test]
fn secure_fill_never_falls_back() {
    let mut bytes = [0; 8];
    random::fill_bytes(&mut bytes);
    assert_eq!(bytes, [0xa5; 8]);
    random::fill_bytes(&mut []);
    FAIL.store(true, Ordering::Relaxed);
    bytes.fill(7);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || random::fill_bytes(&mut bytes))).is_err());
    assert_eq!(bytes, [7; 8]);
}
''')
subprocess.run(['rustc', '--edition=2024', '--test', str(output / 'random-check.rs'),
                '-o', str(output / 'random-check')], check=True)
subprocess.run([str(output / 'random-check')], check=True)
print('Rust PAL records and private open options agree with the C/Native contract')
