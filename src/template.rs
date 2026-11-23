use serde::{Deserialize, Serialize};

use crate::{
    hours::Hours, minutes::Minutes, prepositions::Prepositions, special_cases::SpecialCases,
};

// TODO implement structs properly
#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Template {
    pub language: String,
    pub language_label: String,
    pub hours: Hours,
    pub minutes: Minutes,
    pub prepositions: Prepositions,
    pub special_cases: SpecialCases,
    pub start_sentence: Vec<String>,
    pub end_sentence: Vec<String>,
    #[serde(rename = "on_the_hour_template")]
    pub on_the_hour: Vec<String>,
    #[serde(rename = "normal_template")]
    pub normal: Vec<String>,
    pub next_hour_from: u8,
    #[serde(rename = "add_text")]
    pub addendum: Vec<String>,
}
