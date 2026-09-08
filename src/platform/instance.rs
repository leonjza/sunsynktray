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
        let directory = crate::platform::app_data_dir()?;
        std::fs::create_dir_all(&directory).with_context(|| {
            format!(
                "could not create instance-lock directory {}",
                directory.display()
            )
        })?;
        Self::acquire_at(&directory.join("SunTray.instance.lock"))
    }

    fn acquire_at(path: &PathBuf) -> Result<Option<Self>> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .with_context(|| format!("could not open instance lock {}", path.display()))?;

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self { file })),
            Err(error) if lock_is_unavailable(&error) => Ok(None),
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

fn lock_is_unavailable(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::WouldBlock {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows reports an already-held byte-range/file lock as either
        // ERROR_LOCK_VIOLATION or ERROR_SHARING_VIOLATION rather than
        // ErrorKind::WouldBlock.
        matches!(error.raw_os_error(), Some(32 | 33))
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceLock;

    #[test]
    fn only_one_instance_can_hold_the_lock() {
        let path =
            std::env::temp_dir().join(format!("SunTray.instance.test.{}.lock", std::process::id()));
        let first = InstanceLock::acquire_at(&path)
            .expect("first instance lock should be available")
            .expect("test should own the instance lock");
        assert!(InstanceLock::acquire_at(&path)
            .expect("second lock attempt should succeed")
            .is_none());
        drop(first);
        assert!(InstanceLock::acquire_at(&path)
            .expect("lock should be released after drop")
            .is_some());
        let _ = std::fs::remove_file(path);
    }
}
