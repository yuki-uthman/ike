use std::fs;
use std::fs::File;

use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;

    // filter inactive items
    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.is_active())
        .collect::<Vec<_>>()
        .into();

    // put all the items in a set
    let mut set = std::collections::HashSet::new();
    for item in items.iter() {
        set.insert(item.name());
    }

    let mut groups = vec![
        // disposables
        // "alum",
        "plastic container|round container",
        "plastic bottle",
        "plastic canister",
        "paper bag",
        "paper cup|bowl",
        "(paper|cake|lunch|pizza|burger) box",
        // "food wrap paper",
        // "baggase",
        "\\bstraw\\b",
        // food
        "milkshake",
        // construction
        "steel bar",
        // household items
        "dustpan|broom",
        "knife",
        // "\\bbrush",
        "glass", // drainer, wiper
        "toothbrush",
        "hanger",
        "mop",
        "dustbin",
        "chair",
        "(laundry|shopping|plastic|stand|steel) basket",
        "soklin",
        "lux",
        "wings",
        "\\brack",
        // "tissue paper",
        // "box"
        // "bracket"
        // "shelf"
        // "strip", // store
        // "mattress",
        // "sponge", // contains sponge and sponge mattress

        // "\\bdoor|wpc",
        // "hook",
        // "rack",
        // "shelf",
        // "stand",
        // "tool box",
        // "basket",
        // "tile",
        // "makita",
        // "trolley",
        // "powder",
    ];

    let groups = groups
        .iter_mut()
        .map(|group| {
            let found_items = items.find_all(group).unwrap();

            let mut items = Items::new();

            println!("Group: {}", group);
            for item in found_items.iter() {
                if set.contains(item.name()) {
                    items.add(item.clone());
                    set.remove(item.name());
                } else {
                    println!("already added: {}", item.name());
                    panic!();
                }
            }
            println!("No duplicates!\n");
            (group, items)
        })
        .collect::<Vec<_>>();

    //     // print what is in each group
    //     for group in groups {
    //         println!("Group: {}", group.0);
    //         for item in group.1.clone().into_iter() {
    //             println!("{}", item);
    //         }
    //         println!();
    //     }

    // let mut set = std::collections::HashSet::new();
    // for group in groups {
    //     println!("Group: {}", group.0);
    //     for item in group.1.clone().into_iter() {
    //         let name = item.name().to_string();
    //         if set.contains(name.as_str()) {
    //             println!("duplicate!: {}", item.name());
    //             continue;
    //         }
    //         set.insert(name);
    //         println!("{}", item.name());
    //     }
    //     println!();
    // }

    // to csv
    let dir = "examples/groups";
    // check if dir exists
    if fs::metadata(dir).is_ok() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir(dir).unwrap();

    let mut set = std::collections::HashSet::new();
    for group in groups {
        let group_name = group
            .0
            .replace("|", "_")
            .replace(" ", "_")
            .replace("(", "")
            .replace(")", "")
            .replace("\\b", "");
        let filename = format!("{}/{}.csv", dir, group_name);
        // println!("{}", filename);

        File::create(&filename).unwrap();
        let mut writer = csv::Writer::from_path(filename).unwrap();

        writer
            .write_record(&["Item Name", "Quantity", "Date"])
            .unwrap();

        println!("Group: {}", group.0);
        for item in group.1.clone().into_iter() {
            let name = item.name().to_string();
            if set.contains(name.as_str()) {
                println!("duplicate!: {}", item.name());
                panic!();
            }
            set.insert(name);
            writer
                .write_record(&[item.name(), "0", "01/05/2023"])
                .unwrap();
            println!("{}", item.name());
        }
        println!();

        writer.flush().unwrap();
    }

    Ok(())
}
