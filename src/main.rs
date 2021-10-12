use ktkbot::args;

fn main() {
    let config = args::parse_config();
    ktkbot::run(&config);
}
