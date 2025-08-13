use serde_derive::{
    Deserialize,
    Serialize,
};

use serde_with::serde_as;
use serde_with::formats::PreferOne;
use serde_with::OneOrMany;


fn default_function() -> Option<String> {
    None
}

fn default_targets() -> Vec<String> {
    vec![]
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventSpec {

    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    #[serde(default, alias = "producers")]
    pub producer: Vec<String>,

    #[serde(default, alias = "doc-only")]
    pub doc_only: bool,

    pub producer_ns: Option<String>,

    pub nth: Option<u8>,

    #[serde(default)]
    pub filter: Option<String>,

    #[serde(default)]
    pub rule_name: Option<String>,

    #[serde(default = "default_function")]
    pub function: Option<String>,

    #[serde(default = "default_targets")]
    pub functions: Vec<String>,

    #[serde(default)]
    pub mutation: Option<String>,

    #[serde(default)]
    pub channel: Option<String>,

    #[serde(default)]
    pub state: Option<String>,

    #[serde(default)]
    pub pattern: Option<String>,

    #[serde(default)]
    pub sandboxes: Vec<String>,
}
