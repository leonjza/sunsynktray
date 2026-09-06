#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[cfg(target_os = "macos")]
fn main() {
    use block2::RcBlock;
    use objc2_app_kit::{NSRunningApplication, NSWorkspace, NSWorkspaceOpenConfiguration};
    use objc2_foundation::{ns_string, NSArray, NSDate, NSError, NSRunLoop, NSString, NSURL};

    let helper_executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("SunTray startup helper could not locate itself: {error}");
            return;
        }
    };
    let Some(app_path) = main_app_path(&helper_executable) else {
        eprintln!("SunTray startup helper could not locate SunTray.app");
        return;
    };

    let path = app_path.to_string_lossy();
    let app_path = NSString::from_str(path.as_ref());
    let app_url = NSURL::fileURLWithPath(&app_path);
    let configuration = NSWorkspaceOpenConfiguration::configuration();
    configuration.setArguments(&NSArray::from_slice(&[ns_string!("--startup")]));
    configuration.setActivates(false);
    configuration.setHides(true);
    let completion = RcBlock::new(|_: *mut NSRunningApplication, error: *mut NSError| {
        if !error.is_null() {
            eprintln!("SunTray startup helper could not launch SunTray: {error:p}");
        }
    });
    NSWorkspace::sharedWorkspace().openApplicationAtURL_configuration_completionHandler(
        &app_url,
        &configuration,
        Some(&completion),
    );
    NSRunLoop::currentRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(1.0));
}

#[cfg(target_os = "macos")]
fn main_app_path(executable: &std::path::Path) -> Option<std::path::PathBuf> {
    let helper_app = executable.ancestors().find(|path| {
        path.file_name()
            .is_some_and(|name| name == "SunTrayStartup.app")
    })?;
    let login_items = helper_app
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "LoginItems"))?;
    let library = login_items
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "Library"))?;
    let contents = library
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "Contents"))?;
    contents
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "SunTray.app"))
        .map(std::path::Path::to_owned)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::main_app_path;
    use std::path::Path;

    #[test]
    fn resolves_main_app_from_bundled_helper_path() {
        let path = Path::new(
            "/Applications/SunTray.app/Contents/Library/LoginItems/SunTrayStartup.app/Contents/MacOS/suntray-startup",
        );
        assert_eq!(
            main_app_path(path).unwrap(),
            Path::new("/Applications/SunTray.app")
        );
    }

    #[test]
    fn rejects_unbundled_helper_path() {
        assert!(main_app_path(Path::new("/tmp/suntray-startup")).is_none());
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {}
