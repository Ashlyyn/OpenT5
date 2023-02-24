#[allow(clippy::print_stdout)]
pub fn add_text(i: i32, text: &str) {
    println!("cbuf::add_text: ({}) {}", i, text);
}

pub fn add_textln(i: i32, text: &str) {
    add_text(i, &format!("{}\n", text));
}
