use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    hours::Hours, minutes::Minutes, prepositions::Prepositions, special_cases::SpecialCases,
};

fn de_slot<'de, D: Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    let minute = u8::deserialize(d)?;
    if minute % 5 != 0 || minute > 55 {
        return Err(D::Error::custom(format!(
            "next_hour_from must be a multiple of 5 between 0 and 55, got {minute}"
        )));
    }
    Ok(minute / 5)
}

fn ser_slot<S: Serializer>(slot: &u8, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u8(slot * 5)
}

// TODO implement structs properly
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub(crate) struct Template {
    pub language: String,
    pub language_label: String,
    pub hours: Hours,
    pub minutes: Minutes,
    pub prepositions: Prepositions,
    pub special_cases: SpecialCases,
    pub start_sentence: Vec<String>,
    pub end_sentence: Vec<String>,
    pub on_the_hour_template: Vec<String>,
    pub normal_template: Vec<String>,
    #[serde(
        rename = "next_hour_slot",
        deserialize_with = "de_slot",
        serialize_with = "ser_slot"
    )]
    pub next_hour_from: u8,
}
