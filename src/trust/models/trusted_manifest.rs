use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::TrustedFileRecord;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrustedManifest {
    #[serde(flatten)]
    pub scripts: BTreeMap<String, TrustedFileRecord>,
}
