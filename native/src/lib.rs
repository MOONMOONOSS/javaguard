use neon::prelude::*;

mod openjdk;

static base_open_jdk: &str = "https://api.adoptopenjdk.net/v2/latestAssets/nightly/openjdk";
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

fn latest_open_jdk(mut cx: FunctionContext) -> JsResult<JsValue> {
  use reqwest::blocking;

  let os = String::from(OPERATING_SYS);
  let major: Handle<JsNumber> = match cx.argument_opt(0) {
    Some(val) => {
      if val.is_a::<JsNumber>() {
        val.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
      } else {
        cx.number(8)
      }
    },
    None => cx.number(8),
  };
  let url = format!(
    "{}{}?os={}&arch=x64&heap_size=normal&openjdk_impl=hotspot&type=jre",
    base_open_jdk,
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

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
  Ok(cx.string("hello node"))
}

register_module!(mut cx, {
  cx.export_function("hello", hello);
  cx.export_function("latestOpenJdk", latest_open_jdk)
});
