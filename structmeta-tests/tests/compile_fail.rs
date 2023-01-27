use anyhow::Result;
use ignore::Walk;
use regex::Regex;
use serde::Deserialize;
use std::{
    env::var,
    ffi::OsStr,
    fs::{self},
    path::Path,
    str,
};

#[test]
#[ignore]
fn compile_fail() {
    update_stderr_files("./tests/compile_fail").unwrap();
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*/*.rs")
}

#[derive(Deserialize)]
struct CargoLockRoot {
    package: Vec<CargoLockPackage>,
}

impl CargoLockRoot {
    fn get_version_of(&self, package_name: &str) -> &str {
        for p in &self.package {
            if p.name == package_name {
                return &p.version;
            }
        }
        panic!("pakcage {package_name} not found.");
    }
}

#[derive(Deserialize)]
struct CargoLockPackage {
    name: String,
    version: String,
}

fn update_stderr_files(path: &str) -> Result<()> {
    let manifest_dir = var("CARGO_MANIFEST_DIR")?;
    let manifest_dir = Path::new(&manifest_dir);
    let root_dir = manifest_dir.parent().unwrap();
    let cargo_lock: CargoLockRoot = toml::from_slice(&fs::read(root_dir.join("Cargo.lock"))?)?;
    let syn_version = cargo_lock.get_version_of("syn");

    let path = manifest_dir.join(path);
    for i in Walk::new(path) {
        let i = i?;
        if let Some(file_type) = i.file_type() {
            if file_type.is_file() && i.path().extension() == Some(OsStr::new("stderr")) {
                fix_file(i.path(), syn_version)?;
            }
        }
    }
    Ok(())
}
fn fix_file(path: &Path, syn_version: &str) -> Result<()> {
    let re = Regex::new(r"--> \$CARGO/syn-([0-9]+.[0-9]+.[0-9]+)/")?;
    let b = fs::read(path)?;
    let s0 = str::from_utf8(&b)?;
    let s1 = re.replace_all(s0, format!(r"--> $$CARGO/syn-{syn_version}/"));
    if s0 != s1 {
        fs::write(path, &*s1)?;
    }
    Ok(())
}
