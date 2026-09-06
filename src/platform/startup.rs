use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
const STARTUP_NAME: &str = "SunTray";

pub(crate) fn is_enabled() -> Result<bool> {
    platform::is_enabled()
}

pub(crate) fn set_enabled(enabled: bool) -> Result<()> {
    platform::set_enabled(enabled)
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    pub(super) fn is_enabled() -> Result<bool> {
        let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey(RUN_KEY) {
            Ok(key) => key,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        match key.get_value::<String, _>(STARTUP_NAME) {
            Ok(value) => {
                let executable =
                    std::env::current_exe().context("could not locate SunTray executable")?;
                Ok(value == format!("{} --startup", quote_windows_path(&executable)))
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub(super) fn set_enabled(enabled: bool) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if enabled {
            let (key, _) = hkcu.create_subkey(RUN_KEY)?;
            let executable =
                std::env::current_exe().context("could not locate SunTray executable")?;
            key.set_value(
                STARTUP_NAME,
                &format!("{} --startup", quote_windows_path(&executable)),
            )?;
        } else {
            let key = match hkcu.open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_SET_VALUE) {
                Ok(key) => key,
                Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error.into()),
            };
            match key.delete_value(STARTUP_NAME) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn quote_windows_path(path: &PathBuf) -> String {
        format!("\"{}\"", path.to_string_lossy().replace('"', "\\\""))
    }

    #[cfg(test)]
    mod tests {
        use super::quote_windows_path;
        use std::path::Path;

        #[test]
        fn quotes_paths_with_spaces() {
            assert_eq!(
                quote_windows_path(Path::new(r"C:\Program Files\SunTray.exe").into()),
                r#""C:\Program Files\SunTray.exe""#
            );
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use servicemanagement::{LoginItem, SMAppServiceStatus};

    const LOGIN_ITEM_IDENTIFIER: &str = "com.suntray.startup";

    pub(super) fn is_enabled() -> Result<bool> {
        Ok(matches!(
            LoginItem::new(LOGIN_ITEM_IDENTIFIER)?.status(),
            SMAppServiceStatus::Enabled
        ))
    }

    pub(super) fn set_enabled(enabled: bool) -> Result<()> {
        let app = LoginItem::new(LOGIN_ITEM_IDENTIFIER)?;
        if enabled {
            app.register()
                .context("could not enable SunTray at startup")?;
        } else {
            app.unregister()
                .context("could not disable SunTray at startup")?;
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use super::*;
    pub(super) fn is_enabled() -> Result<bool> {
        Ok(false)
    }
    pub(super) fn set_enabled(_: bool) -> Result<()> {
        anyhow::bail!("startup is not supported on this platform")
    }
}
