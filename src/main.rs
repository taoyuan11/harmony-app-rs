fn main() {
    if let Err(error) = ohos_app::main_entry() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
