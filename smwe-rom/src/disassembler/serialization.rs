use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum LineKind {
    Meta {
        file: String,
    },
    Label {
        label:   String,
        #[serde(skip_serializing_if = "String::is_empty")]
        #[serde(default)]
        comment: String,
    },
    Op {
        op:      String,
        #[serde(skip_serializing_if = "String::is_empty")]
        #[serde(default)]
        arg:     String,
        #[serde(skip_serializing_if = "String::is_empty")]
        #[serde(default)]
        comment: String,
    },
    Empty {},
}
