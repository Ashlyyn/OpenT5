// This file is for any Unix-specific initialization that
// should be done before the rest of main() executes

pub fn main() {
    gtk4::init().unwrap();
    println!("Exiting Unix main()!");
}
