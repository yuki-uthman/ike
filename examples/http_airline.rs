use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Airline {
    id: usize,
    name: String,
    country: String,
    logo: String,
    slogan: String,
    head_quaters: String,
    website: String,
    established: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://api.instantwebtools.net/v1/airlines/1")
        .await?;
    let airline = resp.json::<Airline>().await?;

    println!("{:#?}", airline);
    Ok(())
}
