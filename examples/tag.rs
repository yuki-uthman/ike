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
        .get_active_items();

    let mut groups = [
        // aluminium
        Group::new(
            "alum|ali|baking paper|cling", // todo annotate as cf.group aluminium foil box
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
            "straw",
            &[Tag::Disposable, Tag::Paper, Tag::Restaurant],
            false,
        ),
        Group::new(
            "(dolphine|spice) container|\\b2oz|A500|897",
            &[Tag::Disposable, Tag::Plastic, Tag::Restaurant],
            false,
        ),
        Group::new(
            "baggase|clamshell|suagr cane",
            &[Tag::Disposable, Tag::Baggase, Tag::Restaurant],
            false,
        ),
        Group::new(
            "wood (spoon|fork)|wooden (spoon|fork|ice)", // todo annotate as cutlery
            &[Tag::Disposable, Tag::Wood, Tag::Restaurant],
            false,
        ),
        Group::new(
            "\\brack|mart", // todo annotate as cf.group rack
            &[Tag::Retail],
            false,
        ),
        // shelf
        Group::new("basket|bucket", &[Tag::Restaurant], true),
        Group::new("glass|lg", &[Tag::Restaurant], true),
        Group::new("strip", &[Tag::Restaurant], false),
        Group::new("steel (spoon|fork|soup)", &[Tag::Restaurant], false),
        // hook | bracket
        Group::new("hook|bracket", &[Tag::Construction], false),
        // Construction
        Group::new(
            "plywood|blockboard|deformed|tile|\\bdoor|wpc|paint|spray|makita|cement",
            &[Tag::Construction],
            false,
        ),
        Group::new("plywood|blockboard|tile", &[Tag::Construction], false), // Tag::DIY?
        // cement
        Group::new("board", &[Tag::Construction], false),
        Group::new("tool box", &[Tag::Construction], false), // Tag::DIY?
        // drink
        Group::new(
            "(milkshake|tea|frappe|coffee|smoothie) powder|long beach",
            &[Tag::Restaurant],
            false,
        ),
        Group::new(
            "(carob|cheese|root|onion|sumac|waffle|yogurt|premix|flower) powder",
            &[Tag::Restaurant],
            false,
        ),
        // Household
        Group::new("soklin", &[Tag::Household], false),
        Group::new("wings", &[Tag::Household], false),
        Group::new("lux", &[Tag::Household], false),
        Group::new(
            "dustpan|broom|dustbin|\\bbrush|mop|bucket",
            &[Tag::Household],
            false,
        ),
        Group::new("toothbrush", &[Tag::Household], false),
        Group::new("(iron|ironing) board", &[Tag::Household], false),
        Group::new("hanger|clip", &[Tag::Household], false),
        Group::new("tissue paper", &[Tag::Household], false),
        Group::new("chair", &[Tag::Household], false),
        // sponge
        Group::new("sponge [^f]", &[Tag::Household], false),
        Group::new("mattress", &[Tag::Household], false),
        // luncheon
        Group::new("luncheon", &[Tag::Restaurant], false),
        // cake board
        Group::new("cake board", &[Tag::Restaurant], false),
    ];

    let groups = groups
        .iter_mut()
        .map(|group| {
            let mut items = items.find_all(&group.regex).unwrap();
            // sort by lowercase
            items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
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
            writer.write_record(&[items.name()]).unwrap();
        }
        writer.flush().unwrap();
    }

    Ok(())
}
