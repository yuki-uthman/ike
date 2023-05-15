use shop::invoices::Invoices;

fn main() {
    let invoices = Invoices::new("assets/Invoice.csv");
    println!("{:#?}", invoices)
}
