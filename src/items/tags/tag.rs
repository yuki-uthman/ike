use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

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
    Cutlery,
    Counted,
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
            "cutlery" => Ok(Tag::Cutlery),
            "counted" => Ok(Tag::Counted),
            _ => Err(format!("{} is not a valid tag", s)),
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Tag::Disposable => "disposable",
            Tag::Construction => "construction",
            Tag::Household => "household",
            Tag::Office => "office",
            Tag::Retail => "retail",
            Tag::Restaurant => "restaurant",
            Tag::Aluminium => "aluminium",
            Tag::Steel => "steel",
            Tag::Plastic => "plastic",
            Tag::Paper => "paper",
            Tag::Glass => "glass",
            Tag::Baggase => "baggase",
            Tag::Wood => "wood",
            Tag::PackagedFood => "packaged food",
            Tag::FoodPowder => "food powder",
            Tag::Cutlery => "cutlery",
            Tag::Counted => "counted",
        };
        write!(f, "{}", s)
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        match self {
            Tag::Disposable => "disposable",
            Tag::Construction => "construction",
            Tag::Household => "household",
            Tag::Office => "office",
            Tag::Retail => "retail",
            Tag::Restaurant => "restaurant",
            Tag::Aluminium => "aluminium",
            Tag::Steel => "steel",
            Tag::Plastic => "plastic",
            Tag::Paper => "paper",
            Tag::Glass => "glass",
            Tag::Baggase => "baggase",
            Tag::Wood => "wood",
            Tag::PackagedFood => "packaged food",
            Tag::FoodPowder => "food powder",
            Tag::Cutlery => "cutlery",
            Tag::Counted => "counted",
        }
    }
}
