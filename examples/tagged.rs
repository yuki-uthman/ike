use shop::{Error, Items, Loader, Result, Tag};
use std::fs;
use std::fs::File;

struct Group<'a> {
    pub regex: String,
    tags: &'a [Tag],
    items: Option<Items>,
    should_print: bool,
}

impl<'a> Group<'a> {
    fn new<S>(regex: S, tags: &'a [Tag], should_print: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            regex: regex.into(),
            tags,
            items: None,
            should_print,
        }
    }

    fn add_items(&mut self, mut items: Items) {
        for item in items.iter_mut() {
            item.add_tags(self.tags);
        }
        self.items = Some(items);
    }

    fn print(&self) {
        println!("{}", self.regex);
        for item in self.items.as_ref().unwrap().iter() {
            println!("  {}", item.name());
        }
        println!();
    }
}

fn create_dir(dir: &str) -> Result<()> {
    if fs::metadata(dir).is_ok() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir(dir).unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let items = Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;

    let items = items.only_active_items();

    let mut groups = [
        // aluminium
        Group::new(
            "alum",
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
            &[Tag::Disposable, Tag::Paper, Tag::Retail],
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
            "\\brack\\b|hook|clip|shelf|bracket",
            &[Tag::Retail],
            true,
        ),
        Group::new(
            "wood|spoon|fork",
            &[],
            true,
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

    for group in groups.iter() {
        if group.should_print {
            group.print();
        }

        let filename = format!("{}/{}.csv", dir, group.regex);
        File::create(&filename).unwrap();
        let mut writer = csv::Writer::from_path(filename).unwrap();

        for items in group.items.as_ref().unwrap().iter() {
            writer.serialize(&items).unwrap();
        }
        writer.flush().unwrap();

    }

    Ok(())
}
