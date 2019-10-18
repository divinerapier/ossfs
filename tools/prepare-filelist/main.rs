use clap::{App, Arg};
use std::io::Write;

fn main() {
    let matches = App::new("prepare-filelist")
        .version("1.0")
        .author("divinerapier")
        .about("prepare file list")
        .arg(
            Arg::with_name("directory")
                .required(true)
                .short("d")
                .long("dir")
                .value_name("DIR")
                .help("Sets the target directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .required(true)
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Set the output path")
                .takes_value(true),
        )
        .get_matches();

    let directory = matches.value_of("directory").expect("missing directory");
    let output = matches.value_of("output").expect("missing output");

    let file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(output)
        .expect(&format!("path: {}", output));

    let directory: std::path::PathBuf = std::path::PathBuf::from(directory);

    let mut writer = std::io::BufWriter::new(file);

    for entry in walkdir::WalkDir::new(&directory) {
        let entry: walkdir::DirEntry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            continue;
        }
        writer
            .write_all(
                entry
                    .path()
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        writer.write_all(&vec!['\n' as u8; 1]).unwrap();
    }
}
