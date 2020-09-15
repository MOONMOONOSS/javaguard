use serde::{Serialize, Deserialize};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct LauncherMetadata {
  pub java: JavaAsset,
  pub linux: LinuxAsset,
  pub osx: MacOsAsset,
  pub windows: WindowsAsset,
}

#[derive(Serialize, Deserialize)]
pub struct Asset {
  pub sha1: String,
  pub url: Url,
  pub version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JavaAsset {
  pub lzma: Asset,
  pub sha1: String,
}

#[derive(Serialize, Deserialize)]
pub struct LauncherAsset {
  pub commit: String,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Versions {
  pub launcher: LauncherAsset,
}

#[derive(Serialize, Deserialize)]
pub struct LinuxAsset {
  #[serde(rename = "applink")]
  pub app_link: Url,
  #[serde(rename = "downloadhash")]
  pub download_hash: String,
  pub versions: Versions,
}

#[derive(Serialize, Deserialize)]
pub struct MacOsAsset {
  #[serde(rename = "64")]
  pub x64: JavaAssetFull,
  #[serde(rename = "apphash")]
  pub app_hash: String,
  #[serde(rename = "applink")]
  pub app_link: Url,
  #[serde(rename = "downloadhash")]
  pub download_hash: String,
  pub versions: Versions,
}

#[derive(Serialize, Deserialize)]
pub struct WindowsAsset {
  #[serde(rename = "32")]
  pub x86: JavaAssetFull,
  #[serde(rename = "64")]
  pub x64: JavaAssetFull,
  #[serde(rename = "apphash")]
  pub app_hash: String,
  #[serde(rename = "applink")]
  pub app_link: Url,
  #[serde(rename = "downloadhash")]
  pub download_hash: String,
  #[serde(rename = "rolloutPercent")]
  pub rollout_percent: u16,
  pub versions: Versions,
}

#[derive(Serialize, Deserialize)]
pub struct JavaAssetFull {
  pub jdk: Asset,
  pub jre: Asset,
}
