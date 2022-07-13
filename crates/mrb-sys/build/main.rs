mod presym;

use std::io::Write;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    setup()?;

    let out_dir = Path::new(&std::env::var("OUT_DIR")?).to_path_buf();
    let wrapper_h = write_wrapper_h_file(&out_dir)?;
    println!("cargo:rustc-link-lib=static=mruby");
    generate_bindings(&out_dir, &wrapper_h)?;

    teardown(&out_dir)?;

    Ok(())
}

fn generate_bindings(out_dir: &Path, wrapper_h: &Path) -> Result<(), Box<dyn Error>> {
    let build = bindgen::Builder::default()
        .header(wrapper_h.to_str().unwrap())
        .generate_comments(true)
        .allowlist_file(".*mruby.*")
        .rustified_enum(".*")
        .blocklist_item("presym_name_table")
        .blocklist_item("presym_length_table")
        .use_core()
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    let bindings = if let Some(vendored) = maybe_build() {
        transpile_presym(out_dir, &vendored)?;

        build
            .clang_arg(format!("-I{}", vendored.join("target/include").display()))
            .clang_arg(format!("-I{}", vendored.join("src/include").display()))
            .generate()?
    } else {
        build.generate()?
    };

    bindings.write_to_file(out_dir.join("mruby_bindings.rs"))?;

    Ok(())
}

fn transpile_presym(out_dir: &Path, mruby_dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut outfile = std::fs::File::create(out_dir.join("presym.rs"))?;
    let presym = presym::transpile(&mruby_dir.join("target/include/mruby/presym/table.h"))?;

    writeln!(outfile, "// This file is auto-generated, do not touch.")?;
    writeln!(outfile, "{}", presym)?;

    Ok(())
}

fn setup() -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir("build")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    println!("cargo:rerun-if-env-changed=MRB_SYS_DEBUG_BUILD");

    Ok(())
}

#[cfg(feature = "vendored")]
fn maybe_build() -> Option<PathBuf> {
    mrb_src::Build::new()
        .enable_debug(cfg!(debug_assertions))
        .enable_test(true)
        .enable_bintest(true)
        .build()
        .ok()
}

#[cfg(not(feature = "vendored"))]
fn maybe_build() -> Option<PathBuf> {
    None
}

fn write_wrapper_h_file(out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let wrapper_h = out_dir.join("wrapper.h");
    let mut wrapper_h_file = std::fs::File::create(&wrapper_h)?;
    writeln!(wrapper_h_file, "#define MRB_PRESYM_SCANNING 1")?;
    writeln!(wrapper_h_file, "#include \"mruby.h\"")?;
    writeln!(wrapper_h_file, "#include \"mruby/presym.h\"")?;
    writeln!(wrapper_h_file, "#include \"mruby/presym/id.h\"")?;
    writeln!(wrapper_h_file, "#include \"mruby/presym/table.h\"")?;
    Ok(wrapper_h)
}

fn teardown(_out_dir: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MRB_SYS_DEBUG_BUILD").is_some() {
        eprintln!("MRB_SYS_DEBUG_BUILD is set, exiting");
        std::process::exit(1);
    }

    Ok(())
}
