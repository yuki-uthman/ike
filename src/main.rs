use shop::items::Items;

fn main() {
    let items = Items::new("assets/Item.csv");
    println!("{:#?}", items);
}
