use flate2::read::GzDecoder;
use std::io::Write;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

/// Settings for building mruby.
#[derive(Debug, Clone)]
pub struct Build {
    enable_test: bool,
    enable_bintest: bool,
    enable_debug: bool,
    enable_default_gembox: bool,
    mruby_version: String,
}

impl Build {
    /// Create a new build.
    pub fn new() -> Self {
        Self {
            enable_test: true,
            enable_bintest: true,
            enable_debug: false,
            enable_default_gembox: true,
            mruby_version: "3.1.0".to_string(),
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    /// Enable or disable the test suite.
    pub fn enable_test(&mut self, enabled: bool) -> &mut Self {
        self.enable_test = enabled;
        self
    }

    /// Enable or disable the bintests.
    pub fn enable_bintest(&mut self, enabled: bool) -> &mut Self {
        self.enable_bintest = enabled;
        self
    }

    /// Enable or disable debug build.
    pub fn enable_debug(&mut self, enabled: bool) -> &mut Self {
        self.enable_debug = enabled;
        self
    }

    /// Enable or disable debug build.
    pub fn enable_default_gembox(&mut self, enabled: bool) -> &mut Self {
        self.enable_default_gembox = enabled;
        self
    }

    /// Set the mruby version.
    pub fn mruby_version(&mut self, version: &str) -> &mut Self {
        self.mruby_version = version.to_string();
        self
    }

    /// Build the mruby library from source.
    pub fn build(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        self.setup()?;

        let out_dir = Path::new(&std::env::var("OUT_DIR")?).join(&self.mruby_version);
        let mrb_tarball = self.download_mrb_src(&out_dir)?;
        let mrb_dir = self.unpack_mrb_src(&mrb_tarball, &out_dir)?;
        let build_config = self.generate_build_config(&out_dir)?;
        let dist_dir = self.compile_mrb_src(&mrb_dir, &out_dir, &build_config)?;
        self.print_cargo_flags(&dist_dir);

        self.teardown(&out_dir)?;

        Ok(out_dir)
    }

    fn setup(&self) -> Result<(), Box<dyn Error>> {
        for entry in std::fs::read_dir("build")? {
            let path = entry?.path();

            if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }

        println!("cargo:rerun-if-env-changed=MRB_SRC_DEBUG_BUILD");

        Ok(())
    }

    fn teardown(&self, out_dir: &Path) -> Result<(), Box<dyn Error>> {
        if std::env::var_os("MRB_SRC_DEBUG_BUILD").is_some() {
            eprintln!("MRB_SRC_DEBUG_BUILD is set, exiting");
            std::process::exit(1);
        }

        std::fs::remove_dir_all(out_dir.join("tmp"))?;

        Ok(())
    }

    fn print_cargo_flags(&self, dist_dir: &Path) {
        let target_dir = dist_dir.join("target");

        println!(
            "cargo:rustc-link-search=native={}",
            target_dir.join("lib").display()
        );
        println!("cargo:root={}", target_dir.display());
        println!("cargo:bin={}", dist_dir.join("bin").display());
        println!("cargo:mrbc={}", dist_dir.join("bin/mrbc").display());
        println!("cargo:include={}", target_dir.join("include").display());
        println!("cargo:mruby-version={}", self.mruby_version);
    }

    fn compile_mrb_src(
        &self,
        mrb_dir: &Path,
        out_dir: &Path,
        build_config: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let mut cmd = std::process::Command::new("rake");
        let build_dir = out_dir.to_path_buf();
        let install_dir = build_dir.join("bin");
        eprintln!("Compiling mruby to {}", build_dir.display());
        cmd.env("MRUBY_CONFIG", &build_config);
        cmd.env("MRUBY_BUILD_DIR", &build_dir);
        cmd.env("INSTALL_DIR", &install_dir);
        cmd.stderr(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.current_dir(mrb_dir);
        let output = cmd.output()?;

        if output.status.success() {
            Ok(build_dir)
        } else {
            Err(format!("Compilation failed: {}", output.status).into())
        }
    }

    fn unpack_mrb_src(
        &self,
        mrb_tarball: &Path,
        out_dir: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        eprintln!("Unpacking {}", mrb_tarball.display());

        let mut tar = tar::Archive::new(GzDecoder::new(std::fs::File::open(&mrb_tarball)?));
        let unpack_dir = out_dir.join("tmp");
        let dest_dir = out_dir.join("src");

        match std::fs::remove_dir_all(&dest_dir) {
            Err(_) => eprintln!("Could not clean up {}", dest_dir.display()),
            _ => println!("Cleaned up {}", dest_dir.display()),
        };

        std::fs::create_dir_all(&dest_dir)?;

        tar.unpack(&unpack_dir).unwrap();

        std::fs::rename(
            unpack_dir.join(format!("mruby-{}", self.mruby_version)),
            &dest_dir,
        )?;

        Ok(dest_dir)
    }

    fn download_mrb_src(&self, out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        eprintln!("Downloading mruby to {}", out_dir.display());
        let tmp_dir = out_dir.join("tmp");
        std::fs::create_dir_all(&tmp_dir)?;
        let url = format!(
            "https://github.com/mruby/mruby/archive/refs/tags/{}.tar.gz",
            self.mruby_version
        );
        let dest = tmp_dir.join("src.tar.gz");
        let mut resp = reqwest::blocking::get(url)?;
        let mut out = std::fs::File::create(&dest)?;
        resp.copy_to(&mut out)?;
        Ok(dest)
    }

    fn generate_build_config(&self, out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let build = cc::Build::new();
        let try_compiler = build.try_get_compiler();

        let (compiler, toolchain) = match try_compiler {
            Ok(ref tool) => {
                if tool.is_like_clang() || std::env::var("TARGET")?.contains("apple-darwin") {
                    (Some(tool), "clang")
                } else {
                    (Some(tool), "gcc")
                }
            }
            _ => (None, "gcc"),
        };

        let build_config_path = out_dir.join("build_config.rb");
        let mut config = std::fs::File::create(&build_config_path)?;

        writeln!(config, "MRUBY_VERSION = '{}'", self.mruby_version)?;
        writeln!(config, "MRUBY_TARGET = '{}'", std::env::var("TARGET")?)?;
        writeln!(config, "MRuby::CrossBuild.new('target') do |conf|")?;
        writeln!(config, "  conf.toolchain :{}", toolchain)?;
        writeln!(config, "  conf.host_target = MRUBY_TARGET")?;

        if let Some(compiler) = compiler {
            writeln!(
                config,
                "  conf.cc.command = '{}'",
                compiler.path().display()
            )?;
        }

        if self.enable_bintest {
            writeln!(config, "  conf.enable_bintest")?;
        }

        if self.enable_test {
            writeln!(config, "  conf.enable_test")?;
        }

        if self.enable_default_gembox {
            writeln!(config, "  conf.gembox 'default'")?;
        }

        if self.enable_debug {
            writeln!(config, "  conf.enable_debug")?;
        }

        writeln!(config, "end")?;

        eprintln!("Wrote build config to {}", build_config_path.display());

        Ok(build_config_path)
    }
}
