// use rand::prelude::*;
// use rand_chacha::ChaCha8Rng;

use shop::Items;
use shop::Loader;

fn main() {
    let items = Items::load_from_file("assets/Item.csv").unwrap();
    let item = items.get("Bamboo Stick").unwrap();

    // let mut rng = ChaCha8Rng::seed_from_u64(item.id().try_into().unwrap());
    // println!("{}", rng.gen_range(0..999));
}
