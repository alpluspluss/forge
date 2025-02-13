use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}},
    time::Instant,
};
use std::str::FromStr;
use rayon::prelude::*;
use walkdir::WalkDir;
use log::{info, debug};
use crate::{
    workspace::{Workspace, WorkspaceMember},
    compiler::Compiler,
    cache::BuildCache,
    target::Target,
    toolchains::Toolchain,
    error::{ForgeError, ForgeResult},
};

pub struct Builder {
    workspace: Workspace,
    compiler: Compiler,
    cache: Arc<Mutex<BuildCache>>,
    target_triple: Option<String>,
    selected_profile: Option<String>,
    quick_check: bool,
}

impl Builder {
    pub fn new(
        mut workspace: Workspace,
        target_triple: Option<&str>,
        toolchain_path: Option<&str>,
        sysroot: Option<&Path>,
        profile: Option<&str>,
    ) -> Self {
        let mut cache = BuildCache::new(&workspace.root_path);
        cache.set_quick_check(true);

        let toolchain = target_triple.map(|triple| {
            let target = Target::from_str(triple).expect("Invalid target triple");
            Toolchain::new(
                target,
                toolchain_path,
                sysroot,
                vec![],
            ).expect("Failed to create toolchain")
        });

        let selected_profile = profile.map(String::from);
        workspace.set_profile(selected_profile.clone());
        Builder {
            workspace,
            compiler: Compiler::new(toolchain),
            cache: Arc::new(Mutex::new(cache)),
            target_triple: target_triple.map(String::from),
            selected_profile,
            quick_check: true,
        }
    }

    pub fn build(&self, members: &[&WorkspaceMember]) -> ForgeResult<()> {
        let start = Instant::now();
        info!("Starting build process");

        // Load build cache
        debug!("Loading build cache");
        self.cache.lock().unwrap().load()?;

        // Get build order based on dependencies
        let build_order = self.workspace.get_build_order()?;
        let filtered: Vec<_> = build_order.into_iter()
            .filter(|m| members.is_empty() || members.iter().any(|member| member.name == m.name))
            .collect();

        debug!("Build order: {:?}", filtered.iter().map(|m| &m.name).collect::<Vec<_>>());

        // Build each member
        for member in filtered {
            self.build_member(member)?;
        }

        // Save cache
        debug!("Saving build cache");
        self.cache.lock().unwrap().save()?;

        info!(
            "Build completed in {:.2}s",
            start.elapsed().as_secs_f32()
        );
        Ok(())
    }

    fn build_member(&self, member: &WorkspaceMember) -> ForgeResult<()> {
        let start = Instant::now();
        info!("\nBuilding {}", member.name);

        std::fs::create_dir_all(member.get_build_dir())
            .map_err(|e| ForgeError::Build(format!("Failed to create build directory: {}", e)))?;

        let sources = self.find_sources(member)?;
        info!("Found {} source files", sources.len());

        let target = self.target_triple.as_deref()
            .or_else(|| member.config.cross.as_ref().map(|c| c.target.as_str()))
            .unwrap_or("native");

        let profile = self.selected_profile.as_deref()
            .unwrap_or(&member.config.build.default_profile);

        let profile_config = member.config.get_profile(Some(profile))
            .ok_or_else(|| ForgeError::Build(format!("Profile not found: {}", profile)))?;

        let compiler_flags: Vec<String> = member.config.compiler.flags.iter()
            .chain(profile_config.extra_flags.iter())
            .cloned()
            .collect();

        let total_files = sources.len();
        let completed_files = Arc::new(AtomicUsize::new(0));

        let objects: Vec<PathBuf> = sources.par_iter()
            .map(|source| {
                let object = self.compiler.get_object_path(source, &member.get_build_dir());
                let includes = self.compiler.get_includes(source, &member.get_include_dirs());

                let needs_rebuild = {
                    let cache = self.cache.lock().unwrap();
                    cache.needs_rebuild(
                        source,
                        &object,
                        &includes,
                        &compiler_flags,
                        target,
                        profile
                    )
                };

                if !needs_rebuild {
                    debug!("Skipping {} (up to date)", source.display());
                    let done = completed_files.fetch_add(1, Ordering::SeqCst) + 1;
                    info!("Progress: [{}/{}]", done, total_files);
                    return Ok(object);
                }

                debug!("Compiling {}", source.display());
                self.compiler.compile(
                    source,
                    &object,
                    &member.config.compiler,
                    profile_config,
                    &member.get_include_dirs(),
                    &member.config.build.compiler,
                )?;

                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.update(
                        source,
                        &includes,
                        &compiler_flags,
                        target,
                        profile,
                    )?;
                }

                let done = completed_files.fetch_add(1, Ordering::SeqCst) + 1;
                info!("Progress: [{}/{}]", done, total_files);
                Ok(object)
            })
            .collect::<ForgeResult<_>>()?;

        if !objects.is_empty() {
            info!("Linking {}", member.get_target_path().display());
            self.compiler.link(
                &objects,
                &member.get_target_path(),
                &member.config.compiler,
                profile_config,
                &member.config.build.compiler,
            )?;
        }

        info!(
            "Built {} in {:.2}s",
            member.name,
            start.elapsed().as_secs_f32()
        );
        Ok(())
    }

    fn find_sources(&self, member: &WorkspaceMember) -> ForgeResult<Vec<PathBuf>> {
        let src_dir = member.get_source_dir();
        if !src_dir.exists() {
            return Ok(Vec::new());
        }

        let sources: Vec<_> = WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "cpp" || ext == "c" || ext == "cc")
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        Ok(sources)
    }

    pub fn clean(&self, members: &[&WorkspaceMember]) -> ForgeResult<()> {
        info!("Cleaning workspace");
        for member in members {
            member.clean()?;
        }

        self.cache.lock().unwrap().clean()?;

        info!("Cleaned workspace");
        Ok(())
    }

    pub fn set_quick_check(&mut self, enable: bool) {
        self.quick_check = enable;
        if let Ok(mut cache) = self.cache.lock() {
            cache.set_quick_check(enable);
        }
    }
}