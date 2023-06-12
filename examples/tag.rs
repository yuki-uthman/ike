use shop::{Error, Group, Items, Loader, Result, Tag};
use std::fs;
use std::fs::File;

fn create_dir(dir: &str) -> Result<()> {
    if fs::metadata(dir).is_ok() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir(dir).unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .only_active_items();

    let mut groups = [
        // aluminium
        Group::new(
            "alum", // todo annotate as cf.group aluminium foil box
            &[Tag::Disposable, Tag::Aluminium, Tag::Restaurant],
            false,
        ),
        // plastics
        Group::new(
            "plastic container|round container|square container",
            &[Tag::Disposable, Tag::Plastic, Tag::Restaurant],
            false,
        ),
        Group::new(
            "plastic bottle",
            &[Tag::Disposable, Tag::Plastic, Tag::Household],
            false,
        ),
        Group::new(
            "plastic canister",
            &[Tag::Disposable, Tag::Plastic, Tag::Household],
            false,
        ),
        // paper
        Group::new(
            "paper bag",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant, Tag::Retail],
            false,
        ),
        Group::new(
            "paper cup|bowl",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "paper plate",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "(paper|cake|lunch|pizza|burger) box",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "food wrap paper",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "tissue paper",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant, Tag::Household],
            false,
        ),
        Group::new(
            "paper straw",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "(black|fat) straw",
            &[Tag::Disposable, Tag::Plastic, Tag::Retail],
            false,
        ),
        Group::new(
            "(dolphine|spice) container|\\b2oz|A500|897",
            &[Tag::Disposable, Tag::Plastic, Tag::Restaurant],
            false,
        ),
        Group::new(
            "baggase|clamshell",
            &[Tag::Disposable, Tag::Baggase, Tag::Restaurant],
            false,
        ),
        Group::new(
            "\\brack\\b|hook|clip|shelf|bracket", // todo annotate as cf.group rack
            &[Tag::Retail],
            true,
        ),
        Group::new(
            "wood|spoon|fork", // todo annotate as cutlery
            &[Tag::Restaurant],
            true,
        ),
        // tiles
        Group::new(
            "tile",
            &[Tag::Construction],
            false,
        ),
        // door
        Group::new(
            "\\bdoor|wpc",
            &[Tag::Construction],
            false,
        ),
        // steel bar
        Group::new(
            "steel bar",
            &[Tag::Construction],
            false,
        ),
        // milkshake
        Group::new(
            "milkshake",
            &[Tag::Restaurant],
            false,
        ),
        // soklin
        Group::new(
            "soklin",
            &[Tag::Household],
            false,
        ),
        // wings
        Group::new(
            "wings",
            &[Tag::Household],
            false,
        ),
        // lux
        Group::new(
            "lux",
            &[Tag::Household],
            false,
        ),
        // mop
        Group::new(
            "mop",
            &[Tag::Household],
            false,
        ),
        // "dustpan|broom",
        Group::new(
            "dustpan|broom",
            &[Tag::Household],
            false,
        ),
        // toothbrush
        Group::new(
            "toothbrush",
            &[Tag::Household],
            false,
        ),
        // iron
        Group::new(
            "(iron|ironing) board",
            &[Tag::Household],
            false,
        ),
        // hanger
        Group::new(
            "hanger",
            &[Tag::Household],
            false,
        ),
    ];

    let groups = groups
        .iter_mut()
        .map(|group| {
            let items = items.find_all(&group.regex).unwrap();
            group.add_items(items);
            group
        })
        .collect::<Vec<_>>();

    let dir = "examples/tagged";
    create_dir(dir)?;

    // groups.export("examples/tagged")?;
    for group in groups.iter() {
        if group.should_print {
            group.print();
        }

        let filename = format!("{}/{}.csv", dir, group.regex);
        File::create(&filename).unwrap();
        let mut writer = csv::Writer::from_path(filename).unwrap();

        for items in group.iter() {
            writer.serialize(&items).unwrap();
        }
        writer.flush().unwrap();
    }

    Ok(())
}
