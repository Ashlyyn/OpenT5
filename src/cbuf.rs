pub fn add_text(_: i32, text: String) {
    println!("cbuf::add_text: {}", text);
}

pub fn add_textln(i: i32, text: String) {
    add_text(i, format!("{}\n", text));
}
