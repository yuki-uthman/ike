use shop::Items;
use shop::Loader;

fn main() {
    let mut items = Items::load_from_file("assets/Item.csv")
        .unwrap()
        .get_active_items()
        .get_counted_items();

    // lowercase
    items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    // calculate the value of the items price x quantity
    let mut total_value = 0.0;
    for item in items.iter() {
        let value = item.cost() * item.stock_on_hand() as f32;
        total_value += value;
        println!(
            "{:>8}: {} x {} = {}",
            value,
            item.price(),
            item.stock_on_hand(),
            item.name()
        );
    }

    println!("Total Value: {}", total_value);

    let filename = "examples/output/zakat.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer
        .write_record(&["Name", "Cost", "Quantity", "Total Value"])
        .unwrap();

    for item in items.iter() {
        if item.name().is_empty() {
            continue;
        }
        let value = item.cost() * item.stock_on_hand() as f32;
        writer
            .write_record(&[
                item.name().to_string(),
                item.price().to_string(),
                item.stock_on_hand().to_string(),
                value.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();
}
