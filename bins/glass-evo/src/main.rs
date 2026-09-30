//! The face as a command: Glass's display with the face over it. It takes
//! the same arguments and environment as `glass`, so the plugin starts it
//! in the display's place when glass-evo owns the screen. `--version`
//! says which face this is.

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().skip(1).any(|a| a == "--version" || a == "-V") {
        println!("{}", face::banner());
        return std::process::ExitCode::SUCCESS;
    }
    glass::run_with(args, Some(Box::new(face::Face::new())))
}
