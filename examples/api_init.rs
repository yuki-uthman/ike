use shop::api::Client;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = &args[1];

    let client = Client::read_from("credentials/client.json").unwrap();
    let token = client.get_initial_token(code).await.unwrap();
    // token.write_to("credentials/token.json").unwrap();
}

