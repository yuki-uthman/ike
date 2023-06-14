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

fn only_alphabets_in_the_name(item: &mut Item) -> &Item {
    let name = item
        .name()
        .split(" ")
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .collect::<Vec<_>>()
        .join(" ");
    item.set_name(&name);
    item
}

fn only_misspelled_items(typos: &Vec<String>, mut items: Items) -> Items {
    let items = items
        .iter_mut()
        .filter(|item| {
            let words = item.name().split(" ").collect::<Vec<_>>();

            for word in words {
                if typos.contains(&word.to_string()) {
                    return true;
                }
            }

            false
        })
        .collect::<Vec<_>>();

    items.into()
}

fn get_typos(dict: Dictionary, mut items: Items) -> Vec<String> {
    let items: Items = items
        .iter_mut()
        .map(only_alphabets_in_the_name)
        .collect::<Vec<_>>()
        .into();

    let items = items.get_unique_items();

    let misspelled_names = items
        .iter()
        .filter(|item| {
            if dict.check(&item.name()) {
                return false;
            } else {
                return true;
            }
        })
        .map(|item| item.name().to_owned())
        .collect::<Vec<_>>();

    let mut typos = HashSet::new();
    for name in misspelled_names.iter() {
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
            if typos.contains(&word.to_owned()) {
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

    let typos = get_typos(dict, items.clone().into());

    let misspelled_items = only_misspelled_items(&typos, items.clone().into());
    let items = highlight_typos(&typos, misspelled_items);
    for item in items.iter() {
        println!("{}", item.name());
    }
    Ok(())
}
