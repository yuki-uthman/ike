use shop::api::Token;

fn main() {
    let token = Token::from_file("credentials/token.json").unwrap();
    println!("{} secs remaining", token.expires_in());
}
