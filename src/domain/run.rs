use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RunAction {
    #[default]
    Run,
    Build,
    Debug,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunProfile {
    pub name: String,
    pub command: String,
    pub build_command: Option<String>,
    pub debug_command: Option<String>,
    #[serde(default)]
    pub argument_prefix: String,
    #[serde(default)]
    pub argument_template: Option<String>,
    #[serde(default)]
    pub debug_argument_prefix: String,
    #[serde(default)]
    pub debug_argument_template: Option<String>,
}

impl RunProfile {
    pub fn command_for(&self, action: RunAction) -> Option<&str> {
        match action {
            RunAction::Run => Some(self.command.as_str()),
            RunAction::Build => self.build_command.as_deref(),
            RunAction::Debug => self.debug_command.as_deref(),
        }
        .filter(|command| !command.trim().is_empty())
    }
}

#[derive(Clone, Debug, Default)]
pub struct RunConfiguration {
    pub request: u64,
    pub automatic: bool,
    pub loading: bool,
    pub selected_profile: String,
    pub file: Option<PathBuf>,
    pub command: String,
    pub label: String,
    pub error: Option<String>,
    pub profiles: Vec<RunProfile>,
    pub configurations: Vec<SavedRunConfiguration>,
    pub selected_configuration: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedRunConfiguration {
    pub id: u64,
    pub name: String,
    pub automatic: bool,
    pub file: Option<PathBuf>,
    pub command: String,
    pub selected_profile: String,
    pub profiles: Vec<RunProfile>,
    pub arguments: String,
    pub instances: u16,
    pub allow_parallel: bool,
    #[serde(default)]
    pub error: Option<String>,
}

pub struct RunConfigurationEdit {
    pub id: u64,
    pub name: String,
    pub command: String,
    pub arguments: String,
    pub instances: String,
    pub allow_parallel: bool,
}

#[derive(Clone, Debug)]
pub struct RunInstance {
    pub id: crate::domain::terminal::TerminalId,
    pub configuration: u64,
    pub label: String,
}

#[derive(Clone, Default)]
pub struct RunSessions {
    pub instances: Vec<RunInstance>,
    pub selected: Option<crate::domain::terminal::TerminalId>,
    pub starting: bool,
}

impl RunConfiguration {
    pub fn selected(&self) -> Option<&SavedRunConfiguration> {
        self.configurations
            .iter()
            .find(|config| config.id == self.selected_configuration)
    }
    pub fn current_profile(&self) -> Option<&RunProfile> {
        if !self.automatic || self.loading || self.error.is_some() {
            return None;
        }
        self.profiles
            .iter()
            .find(|profile| profile.name == self.selected_profile)
    }

    pub fn command_for(&self, action: RunAction) -> Option<&str> {
        if self.loading || self.error.is_some() {
            return None;
        }
        if self.automatic {
            self.current_profile()?.command_for(action)
        } else {
            (action == RunAction::Run && !self.command.trim().is_empty())
                .then_some(self.command.as_str())
        }
    }
}
