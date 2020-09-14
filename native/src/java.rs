use std::cmp::Ordering;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Eq)]
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

impl Ord for JavaMeta {
  fn cmp(&self, other: &Self) -> Ordering {
    // Ordering logic on options
    if self.version.is_some() && other.version.is_none() {
      return Ordering::Greater;
    } else if self.version.is_none() && other.version.is_some() {
      return Ordering::Less;
    } else if self.version.is_none() && other.version.is_none() {
      return Ordering::Equal;
    }

    let self_version = self.version.as_ref().unwrap();
    let other_version = other.version.as_ref().unwrap();

    if self_version.major.cmp(&other_version.major) == Ordering::Equal {
      if self_version.update.cmp(&other_version.update) == Ordering::Equal {
        if self_version.build.cmp(&other_version.build) == Ordering::Equal {
          // Same version, give priority to JRE
          if self.exec_path.is_some() && other.exec_path.is_none() {
            return Ordering::Greater;
          } else if self.exec_path.is_none() && other.exec_path.is_some() {
            return Ordering::Less;
          } else if self.exec_path.is_none() && other.exec_path.is_none() {
            return Ordering::Equal;
          }

          let self_exe = self.exec_path.as_ref().unwrap();
          let other_exe = other.exec_path.as_ref().unwrap();

          let self_exe_test = &(
            self_exe
              .to_str()
              .to_owned()
              .expect("Unable to compare self exe path to other exe path")
              .to_lowercase()
            ).contains("jdk");
          
          let other_exe_test = &(
            other_exe
              .to_str()
              .to_owned()
              .expect("Unable to compare other exe path to self exe path")
              .to_lowercase()
            ).contains("jdk");
          
          return self_exe_test.cmp(other_exe_test);
        } else {
          self_version.build.cmp(&other_version.build)
        }
      } else {
        self_version.update.cmp(&other_version.update)
      }
    } else {
      self_version.major.cmp(&other_version.major)
    }
  }
}

impl PartialOrd for JavaMeta {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for JavaMeta {
  fn eq(&self, other: &Self) -> bool {
    if self.version.is_none() && other.version.is_none() {
      true
    } else if self.version.is_some() && other.version.is_some() {
      self.version.as_ref().unwrap() == other.version.as_ref().unwrap()
    } else {
      false
    }
  }
}

pub enum JdkParserError {
  IncompatibleJreVersion,
}

#[derive(Serialize, Deserialize, Debug, Eq)]
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

impl PartialEq for RuntimeVersion {
  fn eq(&self, other: &Self) -> bool {
    self.major == other.major
  }
}
