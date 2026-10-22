use crate::domain::settings::ContextMenuStatus;
use anyhow::{Context as _, Result, ensure};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use windows_registry::{CURRENT_USER, Key};

const OWNER: &str = "gpui_idea.context-menu.v1";
const LABEL: &str = "使用 GPUI Editor 打开";
const ENTRIES: [(&str, &str, &str); 3] = [
    (r"Software\Classes\*\shell\GPUIIdea", "--file", "%1"),
    (
        r"Software\Classes\Directory\shell\GPUIIdea",
        "--project",
        r"%1\.",
    ),
    (
        r"Software\Classes\Directory\Background\shell\GPUIIdea",
        "--project",
        r"%V\.",
    ),
];

// Distinguish missing keys from access/registry errors before any mutation.
fn open(path: &str) -> Result<Option<Key>> {
    match CURRENT_USER.open(path) {
        Ok(key) => Ok(Some(key)),
        Err(error) if matches!(error.code().0 as u32, 0x80070002 | 0x80070003) => Ok(None),
        Err(error) => Err(error).context("无法读取 Windows 右键菜单注册表"),
    }
}

fn check_ownership() -> Result<()> {
    for (path, _, _) in ENTRIES {
        if let Some(key) = open(path)? {
            ensure!(
                key.get_string("GPUIIdeaOwner").ok().as_deref() == Some(OWNER),
                "同名右键菜单属于其他程序，无法覆盖或移除"
            );
        }
    }
    Ok(())
}

fn command(executable: &str, option: &str, argument: &str) -> String {
    // The final dot keeps a drive-root backslash from escaping the closing quote.
    format!("\"{executable}\" {option} \"{argument}\"")
}

fn notify_shell() {
    // No PIDLs are supplied for association changes; the call does not retain data.
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

pub(super) fn status(executable: &str) -> Result<ContextMenuStatus> {
    let mut complete = 0;
    let mut present = 0;
    let icon = format!("\"{executable}\",0");
    for (path, option, argument) in ENTRIES {
        let Some(key) = open(path)? else { continue };
        present += 1;
        let expected = [
            ("GPUIIdeaOwner", OWNER),
            ("MUIVerb", LABEL),
            ("Icon", icon.as_str()),
            ("MultiSelectModel", "Single"),
        ];
        let matches = expected
            .iter()
            .all(|(name, value)| key.get_string(name).is_ok_and(|actual| actual == *value));
        let command_key = open(&format!(r"{path}\command"))?;
        if matches
            && command_key.is_some_and(|key| {
                key.get_string("")
                    .is_ok_and(|actual| actual == command(executable, option, argument))
            })
        {
            complete += 1;
        }
    }
    Ok(ContextMenuStatus {
        supported: true,
        installed: complete == ENTRIES.len(),
        has_entries: present > 0,
    })
}

pub(super) fn install(executable: &str) -> Result<()> {
    ensure!(
        std::path::Path::new(executable).is_file(),
        "程序路径不存在，无法注册右键菜单"
    );
    check_ownership()?;
    let result: Result<()> = (|| {
        for (path, option, argument) in ENTRIES {
            let key = CURRENT_USER
                .create(path)
                .context("无法创建右键菜单注册表项")?;
            key.set_string("GPUIIdeaOwner", OWNER)?;
            key.set_string("MUIVerb", LABEL)?;
            key.set_string("Icon", format!("\"{executable}\",0"))?;
            key.set_string("MultiSelectModel", "Single")?;
            key.create("command")?
                .set_string("", command(executable, option, argument))?;
        }
        Ok(())
    })();
    // A partial failure can still change Explorer's cached menu state.
    notify_shell();
    result.context("无法添加 Windows 右键菜单")
}

pub(super) fn remove() -> Result<()> {
    check_ownership()?;
    let result: Result<()> = (|| {
        for (path, _, _) in ENTRIES {
            if open(path)?.is_some() {
                CURRENT_USER
                    .remove_tree(path)
                    .context("无法移除本程序的右键菜单项")?;
            }
        }
        Ok(())
    })();
    notify_shell();
    result
}
