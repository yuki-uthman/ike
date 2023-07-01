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
            "wood (spoon|fork)|wooden (spoon|fork|ice)", // todo annotate as cutlery
            &[Tag::Disposable, Tag::Wood, Tag::Restaurant],
            false,
        ),
        Group::new(
            "\\brack\\b|hook|clip|shelf|bracket", // todo annotate as cf.group rack
            &[Tag::Retail],
            true,
        ),
        Group::new("steel (spoon|fork|soup)", &[Tag::Restaurant], false),
        // tiles
        Group::new("tile", &[Tag::Construction], false),
        // door
        Group::new("\\bdoor|wpc", &[Tag::Construction], false),
        // steel bar
        Group::new("steel bar", &[Tag::Construction], false),
        // cement
        Group::new("cement", &[Tag::Construction], false),
        // paint
        Group::new("paint|spray", &[Tag::Construction], false),
        Group::new("tool box", &[Tag::Construction], false), // Tag::DIY?
        // makita
        Group::new("makita", &[Tag::Construction], false),
        // drink
        Group::new("(milkshake|tea|frappe|coffee|smoothie) powder|long beach", &[Tag::Restaurant], true),
        Group::new("(carob|cheese|root|onion|sumac|waffle|yogurt|premix|flower) powder", &[Tag::Restaurant], true),

        // Household
        Group::new("soklin", &[Tag::Household], false),
        Group::new("wings", &[Tag::Household], false),
        Group::new("lux", &[Tag::Household], false),
        Group::new("mop", &[Tag::Household], false),
        Group::new("dustpan|broom|dustbin", &[Tag::Household], false),
        Group::new("toothbrush", &[Tag::Household], false),
        Group::new("(iron|ironing) board", &[Tag::Household], false),
        Group::new("hanger", &[Tag::Household], false),
        Group::new("tissue paper", &[Tag::Household], false),
        Group::new("\\bbrush", &[Tag::Household], false),
        Group::new("chair", &[Tag::Household], false),
        Group::new("bucket", &[Tag::Household], false),
        // sponge
        Group::new("sponge [^f]", &[Tag::Household], true),
        Group::new("mattress", &[Tag::Household], true),

        // luncheon
        Group::new("luncheon", &[Tag::Restaurant], false),
        // cake board
        Group::new("cake board", &[Tag::Restaurant], false),

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
            writer.write_record(&[items.name()]).unwrap();
        }
        writer.flush().unwrap();
    }

    Ok(())
}
