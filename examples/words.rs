use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Error> {
    let items = Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;

    let mut map = std::collections::HashMap::new();

    for item in items.iter() {
        let words = item
            .name()
            .split(" ")
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|word| word.chars().all(|c| c.is_alphabetic()))
            .filter(|word| word.len() > 2)
            .map(|word| word.to_lowercase())
            .map(|word| {
                let stop_words = [
                    "and", "the", "for", "with", "from", "this", "that", "these", "those", "size",
                    "with", "only",
                ];
                if stop_words.contains(&word.as_str()) {
                    return "".to_string();
                }
                word.to_string()
            })
            .collect::<Vec<_>>();

        for word in words.into_iter() {
            let count = map.entry(word).or_insert(0);
            *count += 1;
        }
    }

    let filename = "examples/output/words.csv";
    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer.write_record(&["words", "count"]).unwrap();
    for word in map.keys() {
        let count = map.get(word).unwrap();
        writer
            .write_record(&[word.to_string(), count.to_string()])
            .unwrap();
    }
    writer.flush().unwrap();

    Ok(())
}
