use super::association_registry::Key;
use crate::domain::settings::{
    DefaultAppsPage, EDITOR_FILE_EXTENSIONS, FileAssociationAction, FileAssociationStatus,
};
use anyhow::{Context as _, Result, ensure};
use windows::{
    Win32::UI::{
        Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify, ShellExecuteW},
        WindowsAndMessaging::SW_SHOWNORMAL,
    },
    core::{PCWSTR, w},
};
use windows_registry::LOCAL_MACHINE;

const APP: &str = "GPUIIdea";
const PROG_ID: &str = "GPUIIdea.Text.v1";
const PROG_PATH: &str = r"Software\Classes\GPUIIdea.Text.v1";
const CAPABILITIES: &str = r"Software\GPUIIdea\FileAssociations\Capabilities";
const REGISTERED: &str = r"Software\RegisteredApplications";
const OWNER: &str = "gpui_idea.file-associations.v1";

fn entries(executable: &str, extensions: &[String]) -> Vec<(String, String, String)> {
    let mut entries = vec![
        (PROG_PATH.into(), "GPUIIdeaOwner".into(), OWNER.into()),
        (PROG_PATH.into(), "".into(), "GPUI Editor 文本文件".into()),
        (
            format!(r"{PROG_PATH}\DefaultIcon"),
            "".into(),
            format!("\"{executable}\",0"),
        ),
        (
            format!(r"{PROG_PATH}\shell\open\command"),
            "".into(),
            format!("\"{executable}\" --file \"%1\""),
        ),
        (CAPABILITIES.into(), "GPUIIdeaOwner".into(), OWNER.into()),
        (
            CAPABILITIES.into(),
            "ApplicationName".into(),
            "GPUI Editor".into(),
        ),
        (
            CAPABILITIES.into(),
            "ApplicationDescription".into(),
            "使用 GPUI Editor 编辑文本和源代码文件".into(),
        ),
        (
            CAPABILITIES.into(),
            "ApplicationIcon".into(),
            format!("\"{executable}\",0"),
        ),
        (REGISTERED.into(), APP.into(), CAPABILITIES.into()),
    ];
    for extension in extensions {
        entries.push((
            format!(r"{CAPABILITIES}\FileAssociations"),
            extension.clone(),
            PROG_ID.into(),
        ));
        entries.push((
            format!(r"Software\Classes\{extension}\OpenWithProgids"),
            PROG_ID.into(),
            "".into(),
        ));
    }
    entries
}

fn status(entries: &[(String, String, String)]) -> Result<FileAssociationStatus> {
    let mut registered = true;
    for (path, name, expected) in entries {
        let Some(key) = Key::open(path)? else {
            registered = false;
            break;
        };
        if key.get(name)?.as_deref() != Some(expected.as_str()) {
            registered = false;
            break;
        }
    }
    let mut extensions = Vec::new();
    for extension in known_extensions()? {
        if let Some(key) = Key::open(&format!(r"Software\Classes\{extension}\OpenWithProgids"))? {
            if key.get(PROG_ID)?.as_deref() == Some("") {
                extensions.push(extension);
            }
        }
    }
    let mut candidates = EDITOR_FILE_EXTENSIONS
        .iter()
        .map(|extension| (*extension).to_owned())
        .collect::<Vec<_>>();
    for extension in &extensions {
        if !candidates.contains(extension) {
            candidates.push(extension.clone());
        }
    }
    let default_extensions = super::default_associations::query(&candidates, PROG_ID)
        .unwrap_or_else(|error| {
            tracing::warn!(error = %format!("{error:#}"), "default association query failed; no types will be highlighted");
            Vec::new()
        });
    Ok(FileAssociationStatus {
        registered,
        extensions,
        default_extensions,
        requested_settings_page: None,
    })
}

fn known_extensions() -> Result<Vec<String>> {
    let Some(key) = Key::open(&format!(r"{CAPABILITIES}\FileAssociations"))? else {
        return Ok(Vec::new());
    };
    let mut extensions = Vec::new();
    for name in key.value_names()? {
        let Some(suffix) = name.strip_prefix('.') else {
            continue;
        };
        if !suffix.is_empty()
            && suffix.len() <= 64
            && suffix.chars().all(|character| {
                character.is_alphanumeric() || matches!(character, '_' | '-' | '+')
            })
            && key.get(&name)?.as_deref() == Some(PROG_ID)
        {
            extensions.push(name);
        }
    }
    extensions.sort();
    Ok(extensions)
}

fn register(entries: &[(String, String, String)]) -> Result<()> {
    for path in [PROG_PATH, CAPABILITIES] {
        if let Some(key) = Key::open(path)? {
            ensure!(
                key.get("GPUIIdeaOwner")?.as_deref() == Some(OWNER),
                "同名文件关联属于其他程序，无法覆盖"
            );
        }
    }
    if let Some(key) = Key::open(REGISTERED)? {
        ensure!(
            key.get(APP)?.is_none_or(|value| value == CAPABILITIES),
            "同名默认应用已由其他程序注册"
        );
    }
    let result: Result<()> = (|| {
        for (path, name, value) in entries {
            Key::create(path)?.set(name, value)?;
        }
        Ok(())
    })();
    // Notify Explorer even after a partial registration so cached associations are invalidated.
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
    result
}

fn supports_application_page() -> Result<bool> {
    let version = LOCAL_MACHINE.open(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")?;
    let build: u32 = version.get_string("CurrentBuildNumber")?.parse()?;
    // Windows 11 21H2 and 22H2 require the April 2023 cumulative update.
    let supported = match build {
        22000 => version.get_u32("UBR")? >= 1817,
        22621 => version.get_u32("UBR")? >= 1555,
        _ => build >= 22631,
    };
    tracing::debug!(build, supported, "Windows default apps deep link support");
    Ok(supported)
}

fn open_settings_page(page: DefaultAppsPage) -> Result<()> {
    let application_uri;
    let uri = match page {
        DefaultAppsPage::Application => {
            application_uri = format!("ms-settings:defaultapps?registeredAppUser={APP}")
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            PCWSTR(application_uri.as_ptr())
        }
        DefaultAppsPage::Overview => w!("ms-settings:defaultapps"),
    };
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            uri,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    ensure!(
        result.0 as isize > 32,
        "无法打开 Windows 默认应用设置（错误码 {}），请手动进入“设置 → 应用 → 默认应用”",
        result.0 as isize
    );
    Ok(())
}

fn open_settings() -> Result<DefaultAppsPage> {
    let supports_application_page = supports_application_page().unwrap_or_else(|error| {
        tracing::warn!(%error, "could not determine Windows build; opening default apps overview");
        false
    });
    if supports_application_page {
        match open_settings_page(DefaultAppsPage::Application) {
            Ok(()) => return Ok(DefaultAppsPage::Application),
            Err(error) => {
                tracing::warn!(%error, "could not open app-specific default apps settings");
            }
        }
    }
    open_settings_page(DefaultAppsPage::Overview)?;
    Ok(DefaultAppsPage::Overview)
}

pub(super) fn execute(action: FileAssociationAction) -> Result<FileAssociationStatus> {
    let executable = std::env::current_exe().context("无法获取程序路径")?;
    let path = executable.to_str().context("程序路径不是有效的 Unicode")?;
    let path = if let Some(network) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{network}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).to_owned()
    };
    let mut extensions = match &action {
        FileAssociationAction::Refresh => EDITOR_FILE_EXTENSIONS
            .iter()
            .map(|extension| (*extension).to_owned())
            .collect(),
        FileAssociationAction::Configure(extensions)
        | FileAssociationAction::Register(extensions) => extensions.clone(),
    };
    for extension in known_extensions()? {
        if !extensions.contains(&extension) {
            extensions.push(extension);
        }
    }
    let entries = entries(&path, &extensions);
    let registering = !matches!(action, FileAssociationAction::Refresh);
    if registering {
        register(&entries)?;
    }
    let mut status = status(&entries)?;
    if registering {
        ensure!(status.registered, "文件关联注册不完整，请重试");
    }
    if matches!(action, FileAssociationAction::Configure(_)) {
        status.requested_settings_page = Some(open_settings()?);
    }
    tracing::info!(
        ?action,
        registered = status.registered,
        requested_settings_page = ?status.requested_settings_page,
        "Windows file association operation completed"
    );
    Ok(status)
}
