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
        .arg(
            Arg::with_name("threads")
                .required(false)
                .short("t")
                .long("threads")
                .value_name("THREADS")
                .help("Concurrency read")
                .takes_value(true),
        )
        .get_matches();
    let mountpoint = matches.value_of("mountpoint").unwrap();
    if !matches.is_present("recursive") {
        basic(mountpoint.to_owned());
    } else {
        recursive(mountpoint.to_owned(), 32);
    }
}
// total count: 100000, read files: 1387.315472927s, total length: 13609179611
fn recursive(path: String, concurrency: usize) {
    let begin_at = std::time::SystemTime::now();
    // let entries = std::fs::read_dir(path).unwrap();
    let elapsed1 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!("read dir: {:?}", elapsed1);
    let begin_at = std::time::SystemTime::now();
    let wk = walkdir::WalkDir::new(path).into_iter();
    let mut m = vec![];
    for entry in wk {
        // let b = std::time::SystemTime::now();
        let entry: walkdir::DirEntry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            continue;
        }
        let path = entry.path().to_str().unwrap().to_owned();
        m.push(path);
    }
    let elapsed2 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!(
        "list files: total count: {}, elapsed: {:?}",
        m.len(),
        elapsed2,
    );
    let slice = std::sync::Arc::new(m);
    let mut handlers = vec![];
    for i in 0..concurrency {
        let slice = slice.clone();
        let h = std::thread::spawn(move || {
            let mut total_count = 0;
            let mut local_count = 0;
            for index in (i..slice.len()).step_by(concurrency) {
                let data = std::fs::read(&slice[index]).unwrap();
                total_count += data.len();
                local_count += 1;
                if local_count % 10000 == 0 {
                    println!("index: {}, count: {}", i, local_count);
                }
            }
            total_count
        });
        handlers.push(h);
    }
    let mut total_length = 0;
    for h in handlers {
        total_length += h.join().unwrap();
    }
    let elapsed3 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap()
        - elapsed2;
    println!(
        "list files: total count: {}, total length: {}, elapsed: {:?}",
        slice.len(),
        total_length,
        elapsed3,
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
