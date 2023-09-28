#![allow(unused)]
use colored::Colorize;
use shop::{Error, Item, Items, Loader, Result};
use std::{
    collections::HashSet,
    fs::{self, File},
};
use zspell::{DictBuilder, Dictionary};

fn export_typos(dict: Dictionary, items: Items) {
    let mut typos = HashSet::new();
    for item in items.iter() {
        let words = item
            .name()
            .split(" ")
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|word| word.chars().all(|c| c.is_alphabetic()))
            .filter(|word| word.len() > 2)
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>();

        for word in words {
            if !dict.check(&word) {
                typos.insert(word);
            }
        }
    }

    let filename = "examples/output/typos.csv";
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer.write_record(&["typo"]).unwrap();

    let mut words = typos.iter().collect::<Vec<_>>();
    words.sort_by(|a, b| a.cmp(&b));

    for word in words.iter() {
        writer.write_record(&[word]).unwrap();
    }
    writer.flush().unwrap();
}

fn find_typos_in_the_item_name(dict: Dictionary, items: &Items) -> Vec<String> {
    let words = get_all_the_words(items);
    let typos = find_typos(dict, words);

    let mut typos = typos.into_iter().collect::<Vec<_>>();
    typos.sort();
    typos
}

fn get_all_the_words(items: &Items) -> HashSet<String> {
    let mut words_set = HashSet::new();

    for item in items.iter() {
        let words = item
            .name()
            .split(" ")
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|word| word.chars().all(|c| c.is_alphabetic()))
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>();

        for word in words {
            words_set.insert(word);
        }
    }

    words_set
}

fn find_typos(dict: Dictionary, words_set: HashSet<String>) -> HashSet<String> {
    let mut typos = HashSet::new();
    for name in words_set.iter() {
        for word in name.split(" ") {
            if !dict.check(word) {
                typos.insert(word.to_owned());
            }
        }
    }

    typos
}

fn main() -> Result<()> {
    let aff_content =
        fs::read_to_string("assets/dictionary-en/index.aff").expect("failed to load config file");
    let dic_content = fs::read_to_string("assets/dictionary-en/index.dic")
        .expect("failed to load dictionary file");

    let dict: Dictionary = DictBuilder::new()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .expect("failed to build dictionary!");

    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items();

    // items.sort();

    let typos = find_typos_in_the_item_name(dict, &items);

    let is_typo = |item: &&mut Item| -> bool {
        let words = item
            .name()
            .split(" ")
            .into_iter()
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>();

        for word in words {
            if typos.contains(&word.to_string()) {
                return true;
            }
        }

        false
    };

    let mut items_with_typo: Items = items.iter_mut().filter(is_typo).collect::<Vec<_>>().into();

    let highlight_typos = |item: &mut Item| {
        let highlighted_name = item
            .name()
            .split(" ")
            .into_iter()
            .map(|word| {
                if typos.contains(&word.to_lowercase()) {
                    word.red().bold().to_string()
                } else {
                    word.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        item.set_name(&highlighted_name);
    };

    items_with_typo.iter_mut().for_each(highlight_typos);

    println!();
    let mut count = 0;
    for item in items_with_typo.iter() {
        println!("     {}", item.name());
        count += 1;
    }
    println!();
    println!("   Total: {}", count.to_string().red().bold());
    println!();

    Ok(())
}
