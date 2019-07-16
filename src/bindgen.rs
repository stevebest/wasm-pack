//! Functionality related to running `wasm-bindgen`.

use binary_install::Download;
use child;
use command::build::{BuildProfile, Target};
use failure::{self, ResultExt};
use install;
use manifest::CrateData;
use semver;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    bindgen: &Download,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
    profile: BuildProfile,
) -> Result<(), failure::Error> {
    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let out_dir = out_dir.to_str().unwrap();

    let wasm_path = data
        .target_directory()
        .join("wasm32-unknown-unknown")
        .join(release_or_debug)
        .join(data.crate_name())
        .with_extension("wasm");

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = bindgen.binary("wasm-bindgen")?;
    let target_arg = match target {
        Target::Nodejs => "--nodejs",
        Target::NoModules => "--no-modules",
        Target::Web => {
            if supports_web_target(&bindgen_path)? {
                "--web"
            } else {
                bail!("Your current version of wasm-bindgen does not support the 'web' target. Please update your project to wasm-bindgen version >= 0.2.39.")
            }
        }
        Target::Bundler => "--browser",
    };

    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg)
        .arg(target_arg);

    if let Some(value) = out_name {
        cmd.arg("--out-name").arg(value);
    }

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn supports_web_target(cli_path: &PathBuf) -> Result<bool, failure::Error> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.39")?;
    Ok(cli_version >= expected_version)
}
