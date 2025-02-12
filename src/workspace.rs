use std::path::{Path, PathBuf};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: PathBuf,
    //pub root_config: Config,
    pub members: Vec<WorkspaceMember>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub config: Config,
}

impl Workspace {
    pub fn new(root_path: &Path) -> Result<Self, String> {
        let root_config = Config::load(&root_path.join("forge.toml"))?;
        let mut members = Vec::new();

        if !root_config.build.target.is_empty() {
            members.push(WorkspaceMember {
                name: "root".to_string(),
                path: root_path.to_path_buf(),
                config: root_config.clone(),
            });
        }

        for member_name in &root_config.workspace.members {
            if root_config.workspace.exclude.contains(member_name) {
                continue;
            }

            let member_path = root_path.join(member_name);
            let config_path = member_path.join("forge.toml");

            let config = if config_path.exists() {
                Config::load(&config_path)?
            } else {
                Config::default_for_member(member_name)
            };

            members.push(WorkspaceMember {
                name: member_name.clone(),
                path: member_path,
                config,
            });
        }

        Ok(Workspace {
            root_path: root_path.to_path_buf(),
            // root_config,
            members,
        })
    }

    pub fn filter_members(&self, filter: &[String]) -> Vec<&WorkspaceMember> {
        if filter.is_empty() {
            self.members.iter().collect()
        } else {
            self.members
                .iter()
                .filter(|m| filter.contains(&m.name))
                .collect()
        }
    }
}

impl WorkspaceMember {
    pub fn get_source_dir(&self) -> PathBuf {
        self.path.join(&self.config.paths.src)
    }

    pub fn get_include_dir(&self) -> PathBuf {
        self.path.join(&self.config.paths.include)
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.path.join(&self.config.paths.build)
    }

    pub fn get_target_path(&self) -> PathBuf {
        self.get_build_dir().join(&self.config.build.target)
    }
}