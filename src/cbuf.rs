pub fn add_text(_: i32, text: &str) {
    println!("cbuf::add_text: {}", text);
}

pub fn add_textln(i: i32, text: &str) {
    add_text(i, &format!("{}\n", text));
}
