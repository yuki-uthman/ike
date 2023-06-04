use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Tag {
    Disposable,
    Construction,
    Household,
    Office,
    Retail,
    Restaurant,
    Aluminium,
    Steel,
    Plastic,
    Paper,
    Glass,
    Baggase,
    Wood,
    PackagedFood,
    FoodPowder,
}

impl FromStr for Tag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disposable" => Ok(Tag::Disposable),
            "construction" => Ok(Tag::Construction),
            "household" => Ok(Tag::Household),
            "office" => Ok(Tag::Office),
            "retail" => Ok(Tag::Retail),
            "restaurant" => Ok(Tag::Restaurant),
            "aluminium" => Ok(Tag::Aluminium),
            "steel" => Ok(Tag::Steel),
            "plastic" => Ok(Tag::Plastic),
            "paper" => Ok(Tag::Paper),
            "glass" => Ok(Tag::Glass),
            "baggase" => Ok(Tag::Baggase),
            "wood" => Ok(Tag::Wood),
            "packaged food" => Ok(Tag::PackagedFood),
            "food powder" => Ok(Tag::FoodPowder),
            _ => Err(format!("{} is not a valid tag", s)),
        }
    }
}

