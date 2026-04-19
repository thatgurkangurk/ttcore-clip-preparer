use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipsResponse {
    pub clips: Vec<Clip>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSingleClipResponse {
    pub clip: Clip,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub created_by_id: String,
    pub video_id: String,
    pub url: Url,
    pub title: String,
    pub selected: bool,
    pub created_at: String,
    pub overridden_profile_data_id: Option<String>,
    pub creator: Creator,
    pub overridden_profile_data: Option<OverriddenProfileData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverriddenProfileData {
    pub id: String,
    pub line1: String,
    pub line2: String,
}
