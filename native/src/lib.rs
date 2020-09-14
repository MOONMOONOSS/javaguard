use std::path::{Path, PathBuf};
use neon::prelude::*;

mod openjdk;

static BASE_OPEN_JDK: &str = "https://api.adoptopenjdk.net/v2/latestAssets/nightly/openjdk";
static mojang_launcher_meta: &str = "https://launchermeta.mojang.com/mc/launcher.json";

#[cfg(target_os = "windows")]
static REG_KEYS: Vec<&'static str> = vec![
  "\\SOFTWARE\\JavaSoft\\Java Runtime Environment",
  "\\SOFTWARE\\JavaSoft\\Java Development Kit",
];

#[cfg(target_os = "linux")]
static OPERATING_SYS: &str = "linux";

#[cfg(target_os = "windows")]
static OPERATING_SYS: &str = "windows";

#[cfg(target_os = "macos")]
static OPERATING_SYS: &str = "mac";

#[cfg(target_os = "linux")]
fn path_to_java<'a>(root_path: &PathBuf) -> PathBuf {
  PathBuf::from(&format!("{}/bin/java", root_path.display()))
}

#[cfg(target_os = "windows")]
fn path_to_java<'a>(root_path: &PathBuf) -> PathBuf {
  PathBuf::new(format!("{}/bin/javaw.exe", root_path.display()))
}

#[cfg(target_os = "macos")]
fn path_to_java<'a>(root_path: &PathBuf) -> PathBuf {
  PathBuf::new(format!("{}/Contents/Home/bin/java", root_path.display()))
}

fn latest_open_jdk(mut cx: FunctionContext) -> JsResult<JsValue> {
  use reqwest::blocking;

  let os = String::from(OPERATING_SYS);
  let major: Handle<JsNumber> = match cx.argument_opt(0) {
    Some(val) => {
      val.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
    },
    None => cx.number(8),
  };
  let url = format!(
    "{}{}?os={}&arch=x64&heap_size=normal&openjdk_impl=hotspot&type=jre",
    BASE_OPEN_JDK,
    major.value(),
    os,
  );

  Ok(
    neon_serde::to_value(
      &mut cx,
      &blocking::get(&url)
        .unwrap()
        .json::<Vec<openjdk::JreArtifact>>()
        .unwrap()
    )?
  )
}

fn scan_file_system(mut cx: FunctionContext) -> JsResult<JsValue> {
  let arg = cx.argument::<JsString>(0)?;
  let search_str = String::from(&arg.value());
  let search_path = Path::new(&search_str);
  let mut valid_paths: Vec<PathBuf> = vec![];

  if search_path.exists() {
    for entry in search_path.read_dir().expect(&format!("Unable to search {} for entries", search_path.display())) {
      if let Ok(entry) = entry {
        let exe_path = path_to_java(&entry.path());

        if exe_path.exists() {
          valid_paths.push(exe_path);
        }
      }
    }
  }

  Ok(
    neon_serde::to_value(
      &mut cx,
      &valid_paths
    )?
  )
}

register_module!(mut cx, {
  cx.export_function("latestOpenJdk", latest_open_jdk);
  cx.export_function("scanFileSystem", scan_file_system)
});
