use anyhow::{Context, Result};
use fs2::FileExt;
use std::{fs::OpenOptions, path::PathBuf};

/// Holds SunTray's process-wide instance lock for its lifetime.
pub(crate) struct InstanceLock {
    file: std::fs::File,
}

impl InstanceLock {
    /// Attempts to acquire the lock, returning `None` if another instance owns it.
    pub(crate) fn acquire() -> Result<Option<Self>> {
        let path = lock_path();
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("could not open instance lock {}", path.display()))?;

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self { file })),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error)
                .with_context(|| format!("could not lock instance file {}", path.display())),
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn lock_path() -> PathBuf {
    std::env::temp_dir().join("SunTray.instance.lock")
}

#[cfg(test)]
mod tests {
    use super::InstanceLock;

    #[test]
    fn only_one_instance_can_hold_the_lock() {
        let first = InstanceLock::acquire()
            .expect("first instance lock should be available")
            .expect("test should own the instance lock");
        assert!(InstanceLock::acquire()
            .expect("second lock attempt should succeed")
            .is_none());
        drop(first);
        assert!(InstanceLock::acquire()
            .expect("lock should be released after drop")
            .is_some());
    }
}
