use serde::{Serialize, Deserialize};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct JreArtifact {
  pub binary_name: String,
  pub binary_link: Url,
  pub binary_size: u64,
}
