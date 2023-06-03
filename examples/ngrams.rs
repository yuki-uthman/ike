use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Error> {
    use ngrams::Ngram;

    let items = Items::load("assets/zoho/Item.csv").map_err(|source| Error::Load {
        source,
    })?;

    let mut map = std::collections::HashMap::new();
    let items = items
        .iter()
        .map(|item| {
            let words = item
                .name()
                .split(" ")
                .collect::<Vec<_>>()
                .into_iter()
                .filter(|word| word.chars().all(|c| c.is_alphabetic()))
                .filter(|word| word.len() > 2)
                // remove stop words
                .map(|word| word.to_lowercase())
                .map(|word| {
                    let stop_words = [
                        "and", "the", "for", "with", "from", "this", "that", "these", "those",
                        "size", "with", "only",
                    ];
                    if stop_words.contains(&word.as_str()) {
                        return "".to_string();
                    }
                    word.to_string()
                })
                .collect::<Vec<_>>();
            words.join(" ")
        })
        .filter(|item| {
            let words = item.split(" ");
            // count the number of words in the name
            let count = words.clone().count();
            count > 2
        })
        .collect::<Vec<_>>()
        .clone();

    // print item name
    for item in items.iter() {
        if item.contains("with") {
            println!("{}", item);
        }
    }
    //len
    println!("total: {}", items.len());

    for item in items.iter() {
        let ngrams = item
            .split(" ")
            .ngrams(3)
            .collect::<Vec<_>>()
            .iter()
            .filter(|ngram| {
                ngram
                    .iter()
                    .all(|word| word.chars().all(|c| c.is_alphabetic()))
            })
            .filter(|ngram| {
                // filter out ngrams that contain less than 2 words
                ngram.iter().all(|word| word.len() > 2)
            })
            .map(|ngram| ngram.join(" "))
            .map(|ngram| ngram.to_lowercase())
            .collect::<Vec<_>>();

        for gram in ngrams.into_iter() {
            let count = map.entry(gram).or_insert(0);
            *count += 1;
        }
    }

    let filename = "examples/output/ngrams_3(2).csv";
    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer.write_record(&["words", "count"]).unwrap();

    let mut frequncies = map
        .iter()
        .map(|(key, value)| (key, value))
        .collect::<Vec<_>>();

    // sort by value
    frequncies.sort_by(|a, b| b.1.cmp(a.1));

    for word in frequncies.iter() {
        writer
            .write_record(&[&word.0, &word.1.to_string()])
            .unwrap();
    }
    writer.flush().unwrap();

    Ok(())
}
