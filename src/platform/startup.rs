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
            Ok(_) => Ok(true),
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
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
    };

    const LABEL: &str = "com.suntray.app";

    fn plist_path() -> Result<PathBuf> {
        let home = std::env::var_os("HOME").context("HOME is not set")?;
        Ok(PathBuf::from(home)
            .join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist")))
    }

    pub(super) fn is_enabled() -> Result<bool> {
        let path = plist_path()?;
        if !path.exists() {
            return Ok(false);
        }
        // A LaunchAgent can outlive the app bundle it was created from. Do
        // not report startup as enabled when the plist still points at an
        // old copy, otherwise the setting cannot be repaired from the UI.
        let app = app_path()?;
        let expected = plist(&app.to_string_lossy());
        if fs::read_to_string(&path).ok().as_deref() != Some(expected.as_str()) {
            return Ok(false);
        }
        let domain = format!("gui/{}", uid()?);
        let service = format!("{domain}/{LABEL}");
        let status = Command::new("launchctl")
            .args(["print", &service])
            .status()
            .context("could not invoke launchctl")?;
        Ok(status.success())
    }

    pub(super) fn set_enabled(enabled: bool) -> Result<()> {
        let path = plist_path()?;
        if enabled {
            let app = app_path()?;
            fs::create_dir_all(path.parent().context("invalid LaunchAgent path")?)?;
            write_plist(&path, &app)?;
            let domain = format!("gui/{}", uid()?);
            let path_string = path.to_string_lossy().into_owned();
            if let Err(error) = bootout_if_loaded(&domain, &path_string) {
                let _ = fs::remove_file(&path);
                return Err(error);
            }
            let status = Command::new("launchctl")
                .args(["bootstrap", domain.as_str(), path_string.as_str()])
                .status()
                .context("could not invoke launchctl")?;
            if !status.success() {
                // Do not leave a file which makes the next read look enabled
                // when launchd rejected the job.
                let _ = fs::remove_file(&path);
                anyhow::bail!("launchctl could not enable SunTray at startup");
            }
        } else {
            let domain = format!("gui/{}", uid()?);
            let path_string = path.to_string_lossy().into_owned();
            bootout_if_loaded(&domain, &path_string)?;
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn bootout_if_loaded(domain: &str, path: &str) -> Result<()> {
        let service = format!("{domain}/{LABEL}");
        let loaded = Command::new("launchctl")
            .args(["print", &service])
            .status()
            .context("could not invoke launchctl")?;
        if !loaded.success() {
            return Ok(());
        }
        let status = Command::new("launchctl")
            .args(["bootout", domain, path])
            .status()
            .context("could not invoke launchctl")?;
        if !status.success() {
            anyhow::bail!("launchctl could not disable SunTray at startup");
        }
        Ok(())
    }

    fn write_plist(path: &PathBuf, app: &Path) -> Result<()> {
        let temporary = path.with_extension("plist.tmp");
        fs::write(&temporary, plist(&app.to_string_lossy()))?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        Ok(())
    }

    fn app_path() -> Result<PathBuf> {
        let executable = std::env::current_exe().context("could not locate SunTray executable")?;
        let macos_dir = executable
            .parent()
            .context("SunTray executable has no parent directory")?;
        if macos_dir.file_name().and_then(|name| name.to_str()) == Some("MacOS")
            && macos_dir
                .parent()
                .and_then(|path| path.parent())
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".app"))
        {
            return Ok(macos_dir
                .parent()
                .and_then(|path| path.parent())
                .expect("validated app bundle parent")
                .to_owned());
        }
        Ok(executable)
    }

    fn uid() -> Result<String> {
        Command::new("id")
            .arg("-u")
            .output()
            .context("could not determine current user id")
            .and_then(|output| {
                if !output.status.success() {
                    anyhow::bail!("id -u failed");
                }
                Ok(String::from_utf8(output.stdout)?.trim().to_owned())
            })
    }

    fn plist(program: &str) -> String {
        let program = escape_xml(program);
        let arguments = if program.ends_with(".app") {
            format!(
                "<string>/usr/bin/open</string><string>-a</string><string>{program}</string><string>--args</string><string>--startup</string>"
            )
        } else {
            format!("<string>{program}</string><string>--startup</string>")
        };
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>{LABEL}</string>
<key>ProgramArguments</key><array>{arguments}</array>
<key>RunAtLoad</key><true/>
</dict></plist>
            "#
        )
    }

    fn escape_xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    #[cfg(test)]
    mod tests {
        use super::plist;

        #[test]
        fn launch_agent_targets_the_selected_app_bundle() {
            let plist = plist("/Users/test/Sun & Tray.app");
            assert!(plist.contains("/Users/test/Sun &amp; Tray.app"));
            assert!(plist.contains("<string>-a</string>"));
            assert!(plist.contains("<string>--startup</string>"));
        }

        #[test]
        fn launch_agent_targets_a_raw_local_executable_directly() {
            let plist = plist("/Users/test/target/debug/suntray");
            assert!(plist.contains("<string>/Users/test/target/debug/suntray</string>"));
            assert!(!plist.contains("<string>-a</string>"));
        }
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
