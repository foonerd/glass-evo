//! The face as a command. The scaffold answers `--version` and says what it
//! is; the screen comes with the first feature.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{}", face::VERSION);
        return;
    }
    println!("{}: nothing to show yet", face::banner());
}
