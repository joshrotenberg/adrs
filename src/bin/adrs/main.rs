mod cli;

fn main() {
    let matches = cli::build().get_matches();

    dbg!("matches: {:?}", matches);
}
