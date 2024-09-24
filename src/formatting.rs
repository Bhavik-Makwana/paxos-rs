pub fn print_green(text: &str) {
    println!("\x1b[32m{}\x1b[0m", text);
}
pub fn print_red(text: &str) {
    println!("\x1b[31m{}\x1b[0m", text);
}
