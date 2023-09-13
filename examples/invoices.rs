use shop::Invoices;
use shop::Loader;

pub fn main() {
    // let item_name = std::env::args().nth(1).unwrap_or_else(|| {
    //     println!("Usage: invoices <item name>");
    //     std::process::exit(1);
    // });

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    // print name and id
    for invoice in invoices.iter() {
        if invoice.product_id() == 0 {
            println!("{:#?}", invoice);
        }
    }
}
