use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JavaMeta {
  pub arch: Option<u8>,
  pub exec_path: Option<PathBuf>,
  pub version: Option<RuntimeVersion>,
  pub valid: bool,
}

impl JavaMeta {
  pub fn new() -> Self {
    JavaMeta {
      arch: None,
      exec_path: None,
      version: None,
      valid: false,
    }
  }
}

pub enum JdkParserError {
  IncompatibleJreVersion,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuntimeVersion {
  pub build: u16,
  pub update: u16,
  pub major: u8,
}

impl RuntimeVersion {
  pub fn from_ver_string(ver_string: &str) -> Result<Self, JdkParserError> {
    if ver_string.chars().nth(0).expect("Unable to parse version string") == '1' {
      return Err(JdkParserError::IncompatibleJreVersion);
    }

    let parts: Vec<&str> = ver_string.split('-').collect();

    let build = parts[1][2..].trim().parse::<u16>().expect("Unable to parse build number");

    let parts: Vec<&str> = parts[0].split('_').collect();
    let major_vec: Vec<&str> = parts[0].split('.').collect();

    Ok(
      Self {
        build,
        update: parts[1].parse::<u16>().expect("Unable to parse update number"),
        major: major_vec[1].parse::<u8>().expect("Unable to parse major number"),
      }
    )
  }
}
