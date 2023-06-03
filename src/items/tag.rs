use serde::{Deserialize, Serialize};

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
