// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

use crate::io;
use crate::time::{Duration, Instant};

pub(super) fn remove(
    action: impl FnMut() -> io::Result<()>,
    unchanged: impl FnMut() -> io::Result<bool>,
) -> io::Result<()> {
    let start = Instant::now();
    retry(action, unchanged, || {
        if start.elapsed() >= Duration::from_secs(5) {
            return false;
        }
        crate::thread::sleep(Duration::from_millis(1));
        true
    })
}

fn retry(
    mut action: impl FnMut() -> io::Result<()>,
    mut unchanged: impl FnMut() -> io::Result<bool>,
    mut wait: impl FnMut() -> bool,
) -> io::Result<()> {
    loop {
        match action() {
            // Closing a traversal handle retires its authority immediately,
            // but the reaper can still own its node lease. Wait only for the
            // same observed object, keeping remove-if's identity unchanged.
            Err(error) if error.kind() == io::ErrorKind::ResourceBusy => {
                if !unchanged()? || !wait() {
                    return Err(error);
                }
            }
            result => return result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    #[test]
    fn delayed_retirement_allows_removal_after_releasing_the_lease() {
        let leased = Cell::new(true);
        let attempts = Cell::new(0);
        let result = retry(
            || {
                attempts.set(attempts.get() + 1);
                if leased.get() {
                    Err(io::ErrorKind::ResourceBusy.into())
                } else {
                    Ok(())
                }
            },
            || Ok(true),
            || {
                leased.set(false);
                true
            },
        );
        assert!(result.is_ok());
        assert_eq!(attempts.get(), 2);
    }

    #[test]
    fn replacement_is_not_retried_as_the_original_object() {
        let attempts = Cell::new(0);
        let waits = Cell::new(0);
        let result = retry(
            || {
                attempts.set(attempts.get() + 1);
                Err(io::ErrorKind::ResourceBusy.into())
            },
            || Ok(false),
            || {
                waits.set(waits.get() + 1);
                true
            },
        );
        assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::ResourceBusy));
        assert_eq!(attempts.get(), 1);
        assert_eq!(waits.get(), 0);
    }

    #[test]
    fn a_still_owned_lease_is_bounded() {
        let waits = Cell::new(0);
        let result = retry(
            || Err(io::ErrorKind::ResourceBusy.into()),
            || Ok(true),
            || {
                waits.set(waits.get() + 1);
                waits.get() < 3
            },
        );
        assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::ResourceBusy));
        assert_eq!(waits.get(), 3);
    }

    #[test]
    fn replacement_during_wait_keeps_the_original_remove_if_identity() {
        let observed = 17;
        let current = Cell::new(observed);
        let leased = Cell::new(true);
        let attempts = Cell::new(0);
        let result = retry(
            || {
                attempts.set(attempts.get() + 1);
                if current.get() != observed || leased.get() {
                    Err(io::ErrorKind::ResourceBusy.into())
                } else {
                    current.set(0);
                    Ok(())
                }
            },
            || Ok(current.get() == observed),
            || {
                current.set(29);
                leased.set(false);
                true
            },
        );
        assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::ResourceBusy));
        assert_eq!(current.get(), 29);
        assert_eq!(attempts.get(), 2);
    }

    #[test]
    fn other_failures_and_disappearing_objects_are_preserved() {
        let waits = Cell::new(0);
        for failure in [io::ErrorKind::PermissionDenied, io::ErrorKind::NotFound] {
            let result = retry(
                || Err(failure.into()),
                || Ok(true),
                || {
                    waits.set(waits.get() + 1);
                    true
                },
            );
            assert!(matches!(result, Err(error) if error.kind() == failure));
        }
        let result = retry(
            || Err(io::ErrorKind::ResourceBusy.into()),
            || Err(io::ErrorKind::NotFound.into()),
            || {
                waits.set(waits.get() + 1);
                true
            },
        );
        assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::NotFound));
        assert_eq!(waits.get(), 0);
    }

    #[test]
    fn successful_removal_needs_no_identity_read_or_wait() {
        let checks = Cell::new(0);
        let result = remove(
            || Ok(()),
            || {
                checks.set(checks.get() + 1);
                Ok(true)
            },
        );
        assert!(result.is_ok());
        assert_eq!(checks.get(), 0);
    }
}
