use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
use rayon::prelude::*;
use walkdir::WalkDir;
use crate::{
    workspace::{Workspace, WorkspaceMember},
    compiler::Compiler,
    cache::BuildCache,
};

pub struct Builder {
    // workspace: Workspace,
    compiler: Compiler,
    cache: Arc<Mutex<BuildCache>>,
}

impl Builder {
    pub fn new(workspace: Workspace) -> Self {
        let cache = BuildCache::new(&workspace.root_path);
        Builder {
            // workspace,
            compiler: Compiler::new(),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub fn build(&self, members: &[&WorkspaceMember]) -> Result<(), String> {
        let start = Instant::now();

        // Load build cache
        self.cache.lock().unwrap().load()?;

        // Build each member
        for member in members {
            self.build_member(member)?;
        }

        // Save cache
        self.cache.lock().unwrap().save()?;

        println!(
            "Build completed in {:.2}s",
            start.elapsed().as_secs_f32()
        );
        Ok(())
    }

    fn build_member(&self, member: &WorkspaceMember) -> Result<(), String> {
        let start = Instant::now();
        println!("\nBuilding {}", member.name);

        std::fs::create_dir_all(member.get_build_dir())
            .map_err(|e| format!("Failed to create build directory: {}", e))?;

        let sources = self.find_sources(member)?;
        println!("Found {} source files", sources.len());

        /* compiles in parallel */
        let objects = sources.par_iter()
            .map(|source| self.compile_file(source, member))
            .collect::<Result<Vec<_>, _>>()?;

        if !objects.is_empty() {
            self.compiler.link(
                &objects,
                &member.get_target_path(),
                &member.config.build.compiler,
            )?;
        }

        println!(
            "Built {} in {:.2}s",
            member.name,
            start.elapsed().as_secs_f32()
        );
        Ok(())
    }

    fn find_sources(&self, member: &WorkspaceMember) -> Result<Vec<PathBuf>, String> {
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
                    .map_or(false, |ext| ext == "cpp" || ext == "c")
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        Ok(sources)
    }

    fn compile_file(&self, source: &Path, member: &WorkspaceMember) -> Result<PathBuf, String> {
        let object = self.compiler.get_object_path(
            source,
            &member.get_build_dir(),
        );

        let includes = self.compiler.get_includes(
            source,
            &member.get_include_dir(),
        );

        let needs_rebuild = {
            let cache = self.cache.lock().unwrap();
            cache.needs_rebuild(source, &object, &includes)
        };

        if !needs_rebuild {
            println!("Skipping {} (up to date)", source.display());
            return Ok(object);
        }

        // Compile
        self.compiler.compile(
            source,
            &object,
            &member.config.compiler,
            &member.get_include_dir(),
            &member.config.build.compiler,
        )?;

        /* update cache */
        {
            let mut cache = self.cache.lock().unwrap();
            cache.update(source, &includes);
        }

        Ok(object)
    }
}