use crate::{
    config::Config,
    error::{ForgeError, ForgeResult},
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: PathBuf,
    pub root_config: Config,
    pub members: Vec<WorkspaceMember>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub config: Config,
}

impl Workspace {
    pub fn new(root_path: &Path) -> ForgeResult<Self> {
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
            root_config,
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

    /* visitor pattern */
    pub fn get_build_order(&self) -> ForgeResult<Vec<&WorkspaceMember>> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        let mut temp_visited = HashSet::new();

        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for member in &self.members {
            graph.insert(
                member.name.clone(),
                self.root_config.workspace.dependencies
                    .get(&member.name)
                    .cloned()
                    .unwrap_or_default(),
            );
        }

        for member in &self.members {
            if !visited.contains(&member.name) {
                self.visit_member(
                    member,
                    &graph,
                    &mut visited,
                    &mut temp_visited,
                    &mut order,
                )?;
            }
        }

        // Convert names back to member references
        Ok(order)
    }

    fn visit_member<'a>(
        &'a self,
        member: &'a WorkspaceMember,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        order: &mut Vec<&'a WorkspaceMember>,
    ) -> ForgeResult<()> {
        // Check for cycles
        if temp_visited.contains(&member.name) {
            return Err(ForgeError::Workspace(format!(
                "Circular dependency detected involving {}",
                member.name
            )));
        }

        if visited.contains(&member.name) {
            return Ok(());
        }

        temp_visited.insert(member.name.clone());

        // Visit dependencies first
        if let Some(deps) = graph.get(&member.name) {
            for dep_name in deps {
                let dep = self.members
                    .iter()
                    .find(|m| &m.name == dep_name)
                    .ok_or_else(|| ForgeError::Workspace(format!(
                        "Dependency not found: {}",
                        dep_name
                    )))?;

                self.visit_member(dep, graph, visited, temp_visited, order)?;
            }
        }

        temp_visited.remove(&member.name);
        visited.insert(member.name.clone());
        order.push(member);

        Ok(())
    }
}

impl WorkspaceMember {
    pub fn get_source_dir(&self) -> PathBuf {
        self.path.join(&self.config.paths.src)
    }

    pub fn get_include_dirs(&self) -> Vec<PathBuf> {
        self.config.paths.include
            .iter()
            .map(|dir| self.path.join(dir))
            .collect()
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.path.join(&self.config.paths.build)
    }

    pub fn get_target_path(&self) -> PathBuf {
        let mut path = self.get_build_dir();

        if let Some(cross) = &self.config.cross {
            path = path.join(&cross.target);
        }

        if let Some(profile) = self.config.get_profile(None) {
            path = path.join(if profile.debug_info { "debug" } else { "release" });
        }

        path.join(&self.config.build.target)
    }

    pub fn clean(&self) -> ForgeResult<()> {
        if self.get_build_dir().exists() {
            std::fs::remove_dir_all(self.get_build_dir())
                .map_err(|e| ForgeError::Workspace(format!(
                    "Failed to clean build directory: {}",
                    e
                )))?;
        }
        Ok(())
    }
}