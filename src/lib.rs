#[macro_use]
extern crate itertools;
extern crate bio;
extern crate clap;
//use bio::alphabets::dna;
use bio::io::fasta;
use std::collections::HashMap;

use clap::{App, Arg};
use std::{
    env, error::Error, path::{Path, PathBuf},
};

// --------------------------------------------------
type MyResult<T> = Result<T, Box<Error>>;

#[derive(Debug)]
enum FileType {
    Fasta,
    Fastq,
}

// --------------------------------------------------
#[derive(Debug)]
pub struct Config {
    input_file: String,
    format: Option<String>,
    out_dir: PathBuf,
    kmer_size: usize,
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    let buf = PathBuf::from(config.input_file);

    if config.kmer_size > 10 {
        let msg = format!("--kmer_size ({}) must be less than 10", config.kmer_size);
        return Err(From::from(msg));
    }

    if !buf.is_file() {
        return Err(From::from(format!(
            "-f \"{}\" is not a file",
            buf.to_string_lossy()
        )));
    }

    let guessed_format = guess_format(&buf);
    eprintln!("Guessed {:?}", guessed_format);

    let unique_kmers = all_kmers(config.kmer_size);

    println!("seq_id\t{}", unique_kmers.join("\t"));

    //let reader = if format == "fasta" {
    //    fasta::Reader
    //} else if format == "fastq" {
    //    fastq::Reader
    //} else {
    //    return Err(From::from(format!("Unknown --format \"{}\"", format)))
    //}
    //
    //let reader = reader::from_file()?;

    let reader = fasta::Reader::from_file(buf)?;
    for result in reader.records() {
        let record = result?;
        if let Ok(seq) = String::from_utf8(record.seq().to_vec()) {
            let mut count = HashMap::new();
            let kmers = get_kmers(config.kmer_size, &seq.to_uppercase());
            for kmer in &kmers {
                *count.entry(kmer.to_string()).or_insert(0) += 1;
            }

            let mut freq = vec![];
            let n = kmers.len() as f64;
            for kmer in &unique_kmers {
                let cnt = if let Some(c) = &count.get(kmer) {
                    **c as f64
                } else {
                    0 as f64
                };
                freq.push(format!("{}", cnt / n).to_string());
            }
            println!("{}\t{}", record.id(), freq.join("\t"));
        }
    }

    Ok(())
}

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new("Kmer Counter")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@email.arizona.edu>")
        .about("Count kmers per sequence")
        .arg(
            Arg::with_name("input_file")
                .short("-f")
                .long("file")
                .value_name("FILE")
                .help("Input file")
                .required(true),
        )
        .arg(
            Arg::with_name("format")
                .short("-t")
                .long("format")
                .value_name("FORMAT")
                .help("Input file format (fasta/q)")
                .required(false),
        )
        .arg(
            Arg::with_name("kmer_size")
                .short("-k")
                .long("kmer_size")
                .value_name("FORMAT")
                .help("Kmer size")
                .default_value("4"),
        )
        .arg(
            Arg::with_name("out_dir")
                .short("-o")
                .long("outdir")
                .value_name("DIR")
                .help("Output directory")
                .required(false),
        )
        .get_matches();

    let out_dir = match matches.value_of("out_dir") {
        Some(x) => PathBuf::from(x),
        _ => {
            let cwd = env::current_dir()?;
            cwd.join(PathBuf::from("kmer-out"))
        }
    };

    let format = match matches.value_of("format") {
        Some(fmt) => Some(fmt.to_string()),
        _ => None,
    };

    let kmer_size: usize = match matches.value_of("kmer_size") {
        Some(x) => match x.trim().parse() {
            Ok(n) => n,
            _ => 0,
        },
        _ => 0,
    };

    Ok(Config {
        input_file: matches.value_of("input_file").unwrap().to_string(),
        format: format,
        out_dir: out_dir,
        kmer_size: kmer_size,
    })
}

// --------------------------------------------------
fn get_kmers(k: usize, seq: &str) -> Vec<String> {
    let l = seq.len();
    let n = if l >= k { l - k + 1 } else { 0 };
    let mut kmers: Vec<String> = vec![];

    for i in 0..n {
        let kmer = String::from(&seq[i..i + k]);
        let rc = revcomp(&kmer);
        kmers.push(if kmer < rc {
            kmer.to_string()
        } else {
            rc.to_string()
        });
    }

    kmers
}

// --------------------------------------------------
fn revcomp(seq: &str) -> String {
    let mut rc = vec![];
    for c in seq.chars().rev() {
        let r = match c {
            'A' => 'T',
            'C' => 'G',
            'G' => 'C',
            'T' => 'A',
            _ => 'X',
        };
        rc.push(r);
    }

    rc.into_iter().collect()
}

// --------------------------------------------------
fn kproduct(seq: String, k: usize) -> Vec<String> {
    match k {
        0 => vec![],
        1 => seq.chars().map(|c| c.to_string()).collect(),
        2 => iproduct!(seq.chars(), seq.chars())
            .map(|(a, b)| format!("{}{}", a, b))
            .collect(),
        _ => iproduct!(kproduct(seq.clone(), k - 1), seq.chars())
            .map(|(a, b)| format!("{}{}", a, b))
            .collect(),
    }
}

// --------------------------------------------------
fn all_kmers(n: usize) -> Vec<String> {
    kproduct(String::from("ACGT"), n).into_iter().collect()
}

// --------------------------------------------------
fn guess_format(buf: &PathBuf) -> Option<FileType> {
    if let Some(ext) = Path::new(&buf).extension() {
        if let Some(s) = ext.to_str() {
            match s {
                "fa" => Some(FileType::Fasta),
                "fasta" => Some(FileType::Fasta),
                "fna" => Some(FileType::Fasta),
                "fastq" => Some(FileType::Fastq),
                "fq" => Some(FileType::Fastq),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}
