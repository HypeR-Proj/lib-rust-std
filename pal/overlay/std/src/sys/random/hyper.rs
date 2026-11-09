// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

pub fn fill_bytes(bytes: &mut [u8]) {
    // SAFETY: the shim writes only this uniquely borrowed slice and retains no
    // pointer; startup authority and source readiness are checked by the runtime.
    let status =
        unsafe { crate::sys::pal::ffi::__hyper_std_random_fill(bytes.as_mut_ptr(), bytes.len()) };
    if status != 0 {
        panic!("secure random service failed: {status}");
    }
}
