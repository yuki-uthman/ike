use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Error> {
    use ngrams::Ngram;

    let items = Items::load("assets/zoho/Item.csv").map_err(|source| Error::Load {
        filename: "assets/zoho/Item.csv",
        source,
    })?;

    let mut map = std::collections::HashMap::new();

    for item in items.iter() {
        let ngrams = item
            .name()
            .split(" ")
            .ngrams(2)
            .collect::<Vec<_>>()
            .iter()
            .filter(|ngram| {
                ngram
                    .iter()
                    .all(|word| word.chars().all(|c| c.is_alphabetic()))
            })
            .filter(|ngram| {
                // filter out ngrams that contain less than 2 words
                ngram
                    .iter()
                    .all(|word| word.len() > 2)
            })
            .map(|ngram| ngram.join(" "))
            .map(|ngram| ngram.to_lowercase())
            .collect::<Vec<_>>();

        for gram in ngrams.into_iter() {
            let count = map.entry(gram).or_insert(0);
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
