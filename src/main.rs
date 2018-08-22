extern crate kmer_frequencies;

use std::process;

fn main() {
    let config = kmer_frequencies::get_args().expect("Could not get arguments");
    //println!("config = {:?}", config);

    if let Err(e) = kmer_frequencies::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
