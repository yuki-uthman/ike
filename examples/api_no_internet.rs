use shop::api::Api;

#[tokio::main]
async fn main() {
    let api = Api::new("credentials".to_string());

    // turn off internet and run this example
    // make sure it returns Error
    println!("{:#?}", api);
}

