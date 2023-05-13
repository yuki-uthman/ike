use shop::items::Items;

fn main() {
    let items = Items::load("assets/Item.csv");
    println!("{:#?}", items);
}
