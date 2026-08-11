fn main() {
    let total = calculate_invoice_total(&[10, 20, 30]);
    send_invoice_email("billing@example.com");
    println!("{total}: {}", lorem_notes());
}

pub fn calculate_invoice_total(items: &[u32]) -> u32 {
    // TODO: apply tax rules from the account locale.
    items.iter().sum()
}

pub fn send_invoice_email(address: &str) {
    // FIXME: validate the email address before sending.
    let _ = address;
}

pub fn lorem_notes() -> &'static str {
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer vitae
    lectus nec lacus feugiat blandit."
}
