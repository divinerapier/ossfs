use clap::{App, Arg};

fn main() {
    let matches = App::new("readfiles")
        .version("1.0")
        .author("divinerapier")
        .about("benchmark readfiles")
        .arg(
            Arg::with_name("mountpoint")
                .required(true)
                .short("m")
                .long("mountpoint")
                .value_name("MOUNT_POINT")
                .help("Sets a custom mountpoint")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("recursive")
                .required(false)
                .short("r")
                .long("recursive")
                .value_name("RECURSIVE")
                .help("Read recursively")
                .takes_value(false),
        )
        .get_matches();
    let mountpoint = matches.value_of("mountpoint").unwrap();
    if !matches.is_present("recursive") {
        basic(mountpoint.to_owned());
    } else {
        recursive(mountpoint.to_owned());
    }
}

fn recursive(path: String) {
    let begin_at = std::time::SystemTime::now();
    // let entries = std::fs::read_dir(path).unwrap();
    let elapsed1 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!("read dir: {:?}", elapsed1);
    let mut total_length = 0;
    let begin_at = std::time::SystemTime::now();
    let mut total_count = 0;
    let wk = walkdir::WalkDir::new(path).into_iter();
    for (index, entry) in wk.enumerate() {
        let b = std::time::SystemTime::now();
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            total_count = index + 1;
            continue;
        }
        let data = std::fs::read(entry.path()).unwrap();
        total_length += data.len();
        let e = std::time::SystemTime::now().duration_since(b).unwrap();
        if index % 1000 == 0 {
            println!("read file: {:?}", e);
        }
        if index % 10000 == 0 {
            println!("count: {}", index);
        }
        total_count = index + 1;
    }
    let elapsed2 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!(
        "total count: {}, read files: {:?}, total length: {}",
        total_count, elapsed2, total_length
    );
}

fn basic(path: String) {
    let begin_at = std::time::SystemTime::now();
    let entries = std::fs::read_dir(path).unwrap();
    let elapsed1 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!("read dir: {:?}", elapsed1);
    let mut total_length = 0;
    let begin_at = std::time::SystemTime::now();
    // let mut m = std::collections::HashSet::new();
    let mut total_count = 0;
    for (index, entry) in entries.enumerate() {
        let b = std::time::SystemTime::now();
        let entry: std::fs::DirEntry = entry.unwrap();
        // if m.contains(&entry.path()) {
        //     println!("duplicate key: {:?}", entry.path());
        //     continue;
        // }
        // println!("{:?}", entry.path());
        // m.insert(entry.path());
        if entry.metadata().unwrap().is_file() {
            let data = std::fs::read(entry.path()).unwrap();
            total_length += data.len();
        }
        let e = std::time::SystemTime::now().duration_since(b).unwrap();
        if index % 1000 == 0 {
            println!("read file: {:?}", e);
        }
        if index % 10000 == 0 {
            println!("count: {}", index);
        }
        total_count = index + 1;
    }
    let elapsed2 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!(
        "total count: {}, read files: {:?}, total length: {}",
        total_count, elapsed2, total_length
    );
}
