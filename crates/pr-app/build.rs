fn main() {
    if let Err(hiba) = slint_build::compile("ui/app.slint") {
        eprintln!("A Slint-felület fordítása sikertelen: {hiba}");
        std::process::exit(1);
    }
}
