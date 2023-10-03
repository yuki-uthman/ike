use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut items = Items::load_from_file("assets/Item.csv")?
        .get_active_items()
        .get_counted_items()
        .get_non_combo_items()
        .get_has_stock_items();

    // lowercase
    items.sort_by_name();

    // calculate the value of the items price x quantity
    let mut total_value = 0.0;
    for item in items.iter() {
        let value = item.cost() * item.stock_on_hand() as f32;
        total_value += value;
        println!(
            "{:>10}: MVR{:>5} x {:>5} {}",
            value as u32,
            item.price() as u32,
            item.stock_on_hand(),
            item.name()
        );
    }

    println!("Total Value: {}", total_value);

    let filename = "examples/output/zakat.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer
        .write_record(["Name", "Cost", "Quantity", "Total Value"])
        .unwrap();

    for item in items.iter() {
        if item.name().is_empty() {
            continue;
        }
        let value = item.cost() * item.stock_on_hand() as f32;
        writer
            .write_record([
                item.name().to_string(),
                item.price().to_string(),
                item.stock_on_hand().to_string(),
                value.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();

    Ok(())
}
