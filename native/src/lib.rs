use std::path::{Path, PathBuf};
use neon::prelude::*;

mod java;
mod openjdk;

static BASE_OPEN_JDK: &str = "https://api.adoptopenjdk.net/v2/latestAssets/nightly/openjdk";
static mojang_launcher_meta: &str = "https://launchermeta.mojang.com/mc/launcher.json";

#[cfg(target_os = "windows")]
static REG_KEYS: Vec<&'static str> = vec![
  "SOFTWARE\\JavaSoft\\Java Runtime Environment",
  "SOFTWARE\\JavaSoft\\Java Development Kit",
];

#[cfg(target_os = "linux")]
static OPERATING_SYS: &str = "linux";

#[cfg(target_os = "windows")]
static OPERATING_SYS: &str = "windows";

#[cfg(target_os = "macos")]
static OPERATING_SYS: &str = "mac";

#[cfg(any(target_os = "macos", target_os = "linux"))]
static JAVA_BIN_NAME: &str = "java";

#[cfg(target_os = "windows")]
static JAVA_BIN_NAME: &str = "java.exe";

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

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_java_exe_path(path: &PathBuf) -> bool {
  path.ends_with("bin/java")
}

#[cfg(target_os = "windows")]
fn is_java_exe_path(path: &PathBuf) -> bool {
  path.ends_with("bin/javaw.exe")
}

#[cfg(target_os = "windows")]
fn scan_registry() {
  use winreg::enums::*;
  use winreg::RegKey;
  
  let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

  for entry in REG_KEYS {
    match hklm.open_subkey(entry) {
      Ok(val) => {
        for (name, value) in val.enum_values().map(|x| x.expect("Unable to enumerate registry keys")) {
          println!("{:#?}", name);
          println!("{:#?}", value);
        }
      }
    }
  }
}

fn scan_java_home() -> Option<PathBuf> {
  let home = std::env::var("JAVA_HOME");

  if let Ok(path) = home {
    Some(PathBuf::from(path))
  } else {
    None
  }
}

fn validate_jvm_properties(stderr: &str) -> java::JavaMeta {
  let mut meta = java::JavaMeta::new();

  for line in stderr.split('\n') {
    if line.contains("sun.arch.data.model") {
      let line_vec: Vec<&str> = line.trim().split('=').collect();
      let arch: u8 = line_vec[1][1..].parse::<u8>().expect("Could not determine System Architecture!");

      if arch == 64u8 {
        meta.arch = Some(arch);
      }
    } else if line.contains("java.runtime.version") {
      let version_vec: Vec<&str> = line.trim().split('=').collect();
      let version_str = version_vec[1];

      match java::RuntimeVersion::from_ver_string(version_str) {
        Ok(val) => {
          if val.major == 8u8 && val.update > 52u16 {
            meta.version = Some(val);
          }
        },
        _ => {},
      }
    }
  }

  meta
}

fn validate_java_binary(bin_path: &mut PathBuf) -> java::JavaMeta {
  use std::process::{Command, Stdio};
  use std::str;

  if !is_java_exe_path(&bin_path) {
    return java::JavaMeta::new();
  } else if bin_path.exists() {
    // javaw.exe no longer outputs this information
    // so we use java.exe instead

    bin_path.pop();
    bin_path.push(JAVA_BIN_NAME);

    let childs_first_words = Command::new(bin_path)
      .arg("-XshowSettings:properties")
      .stderr(Stdio::piped())
      .output()
      .expect("Unable to start JVM!");
    
    validate_jvm_properties(str::from_utf8(&childs_first_words.stderr).expect("The baby did not speak"))
  } else {
    java::JavaMeta::new()
  }
}

fn validate_root_vec(root: &mut Vec<PathBuf>) -> Vec<java::JavaMeta> {
  let mut valid: Vec<java::JavaMeta> = vec![];

  for entry in root.iter() {
    let mut entry_buf = entry.to_path_buf();
    let mut meta = validate_java_binary(&mut entry_buf);

    if meta.arch.is_some() && meta.version.is_some() {
      meta.exec_path = Some(entry_buf);
      meta.valid = true;

      valid.push(meta);
    }
  }

  valid
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

fn _scan_file_system(arg: String) -> Vec<PathBuf> {
  let search_path = Path::new(&arg);
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

  valid_paths
}

fn scan_file_system(mut cx: FunctionContext) -> JsResult<JsValue> {
  let arg = cx.argument::<JsString>(0)?;
  
  Ok(
    neon_serde::to_value(
      &mut cx,
      &_scan_file_system(arg.value())
    )?
  )
}

#[cfg(target_os = "linux")]
fn java_validate(mut cx: FunctionContext) -> JsResult<JsValue> {
  let data_dir_arg = cx.argument::<JsString>(0)?;
  let data_dir = String::from(data_dir_arg.value());
  let mut super_set: Vec<PathBuf> = vec![];
  super_set.extend(_scan_file_system("/usr/lib/jvm".to_owned())
    .iter()
    .cloned()
  );
  super_set.extend(_scan_file_system(format!("{}/runtime/x64", data_dir))
    .iter()
    .cloned()
  );

  let jhome = scan_java_home();

  if let Some(path) = jhome {
    super_set.push(path);
  }

  let mut root_sets = validate_root_vec(&mut super_set);
  root_sets.sort();
  
  Ok(
    neon_serde::to_value(
      &mut cx,
      &root_sets
    )?
  )
}

#[cfg(target_os = "windows")]
fn java_validate(mut cx: FunctionContext) -> JsResult<JsValue> {
  let data_dir_arg = cx.argument::<JsString>(0)?;
  let data_dir = String::from(data_dir_arg.value());
  let mut super_set: Vec<PathBuf> = vec![];
  scan_registry();
  super_set.extend(_scan_file_system("C:\\Program Files\\Java".to_owned())
    .iter()
    .cloned()
  );
  super_set.extend(_scan_file_system(format!("{}\\runtime\\x64", data_dir))
    .iter()
    .cloned()
  );

  let jhome = scan_java_home();

  if let Some(path) = jhome {
    if !&(path.to_str().to_owned().to_lowercase()).contains("(x86)") {
      super_set.push(path);
    }
  }

  let mut root_sets = validate_root_vec(&mut super_set);
  root_sets.sort();
  
  Ok(
    neon_serde::to_value(
      &mut cx,
      &root_sets
    )?
  )
}

register_module!(mut cx, {
  cx.export_function("latestOpenJdk", latest_open_jdk);
  cx.export_function("scanFileSystem", scan_file_system);
  cx.export_function("javaValidate", java_validate)
});
