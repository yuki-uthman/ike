use shop::api::Api;
use shop::api::Error;
use colored::Colorize;

#[tokio::main]
async fn main() {
    let api = Api::new("credentials".to_string());

    if let Err(ref error) = api {
        match error {
            Error::NoInternetConnection => {
                println!();
                println!("   {}", "No internet connection".red());
                println!();
            }
            Error::NotInitialized => {
                println!();
                println!("   {}", "API has not been initialized".red());
                println!("   {} {}", "Please run".red(), "init".green().bold());
                println!();
            }
        }
        return;
    }

    let mut api = api.unwrap();

    println!("{:#?}", api.token());

    api.refresh_access_token().await;

    println!("{:#?}", api.token());

}


