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

fn get_typos(dict: Dictionary, items: &Items) -> Vec<String> {
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

    let mut typos = HashSet::new();
    for name in words_set.iter() {
        for word in name.split(" ") {
            if !dict.check(word) {
                typos.insert(word.to_owned());
            }
        }
    }

    let mut typos = typos.into_iter().collect::<Vec<_>>();
    typos.sort();
    typos
}

fn highlight_typos(typos: &Vec<String>, items: Items) -> Items {
    let mut items = items;
    for item in items.iter_mut() {
        let name = item.name();
        let mut new_name = String::new();
        for word in name.split(" ") {
            if typos.contains(&word.to_owned().to_lowercase()) {
                new_name.push_str(&word.red().bold().to_string());
            } else {
                new_name.push_str(word);
            }
            new_name.push_str(" ");
        }
        item.set_name(&new_name);
    }
    items
}

fn main() -> Result<()> {
    let aff_content = fs::read_to_string("node_modules/dictionary-en/index.aff")
        .expect("failed to load config file");
    let dic_content = fs::read_to_string("node_modules/dictionary-en/index.dic")
        .expect("failed to load dictionary file");

    let dict: Dictionary = DictBuilder::new()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .expect("failed to build dictionary!");

    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items();
    items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    let typos = get_typos(dict, &items);
    println!("{:#?}", typos);

    let misspelled = |item: &&mut Item| -> bool {
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

    let misspelled_items = items
        .iter_mut()
        .filter(misspelled)
        .collect::<Vec<_>>()
        .into();

    let items = highlight_typos(&typos, misspelled_items);
    for item in items.iter() {
        println!("{}", item.name());
    }
    Ok(())
}
