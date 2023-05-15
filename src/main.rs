use shop::records::Records;
use shop::invoices::Invoices;

fn main() {
    println!("{:#?}", invoices)
    let invoices = Invoices::load("assets/Invoice.csv").unwrap();
}
