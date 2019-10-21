use clap::{App, Arg};
use ossfs::Counter;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() {
    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or(std::env::var("LOG_LEVEL").unwrap_or(String::from("debug"))),
    )
    .init();
    log::set_max_level(log::LevelFilter::max());

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
        .arg(
            Arg::with_name("max-keys")
                .required(false)
                .long("max-keys")
                .value_name("MAX_KEYS")
                .help("Max keys to read")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("shuffle")
                .required(false)
                .long("shuffle")
                .help("Shuffle file list")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("count")
                .required(false)
                .long("count")
                .short("c")
                .value_name("COUNT")
                .help("Loop count")
                .takes_value(true),
        )
        .get_matches();
    let mountpoint = matches.value_of("mountpoint").unwrap();
    let max_keys = if matches.is_present("max-keys") {
        matches
            .value_of("max-keys")
            .unwrap()
            .parse::<i64>()
            .unwrap()
    } else {
        -1
    };
    let count: usize = if matches.is_present("count") {
        matches.value_of("count").unwrap().parse::<usize>().unwrap()
    } else {
        1
    };
    println!("count: {}", count);
    let shuffle = matches.is_present("shuffle");
    if !matches.is_present("recursive") {
        // basic(mountpoint.to_owned(), max_keys);
    } else {
        recursive(mountpoint.to_owned(), 32, max_keys, shuffle, count);
    }
}
// total count: 100000, read files: 1387.315472927s, total length: 13609179611
fn recursive(path: String, concurrency: usize, max_keys: i64, shuffle: bool, count: usize) {
    let begin_at = SystemTime::now();
    let mut m = vec![];
    for entry in walkdir::WalkDir::new(path).into_iter() {
        let entry: walkdir::DirEntry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            continue;
        }
        let path = entry.path().to_str().unwrap().to_owned();
        m.push(path);
        let length = m.len();
        if length % 100000 == 0 {
            println!("load {}", length);
        }
        if max_keys > 0 && length >= max_keys as usize {
            break;
        }
    }
    let elapsed2 = SystemTime::now().duration_since(begin_at).unwrap();
    println!("load file list: {:?}", elapsed2);
    if shuffle {
        srand::ThreadLocal::seed(
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
        srand::ThreadLocal::shuffle(&mut m);
        let shuffle_elapsed = std::time::SystemTime::now()
            .duration_since(begin_at)
            .unwrap();
        println!("shuffle {:?}", shuffle_elapsed - elapsed2);
    }
    let mut slice = vec![];
    println!("count: {}", count);
    for _ in 0..count {
        // slice.append(&mut m);
        slice.extend_from_slice(&m);
    }
    println!("slice.len() = {}", slice.len());
    let slice = Arc::new(slice);
    let counter = Counter::new(1);
    let mut total_length = 0usize;
    let global_index = Arc::new(AtomicUsize::new(0));
    let mut handlers = vec![];
    for i in 0..concurrency {
        let slice = slice.clone();
        let global_index = global_index.clone();
        let counter = counter.clone();
        let h = std::thread::spawn(move || {
            let mut total_count = 0usize;
            loop {
                let index = global_index.fetch_add(1, Ordering::SeqCst);
                if index >= slice.len() {
                    return total_count;
                }
                let _start = counter.start("read".to_owned());
                let data = std::fs::read(&slice[index]).unwrap();
                total_count += data.len();
                if (index + 1) % 10000 == 0 {
                    let time_now = SystemTime::now();
                    let now = time_now.duration_since(UNIX_EPOCH).unwrap();
                    let e: Duration = time_now.duration_since(begin_at).unwrap() - elapsed2;
                    let e = e.as_secs_f64();
                    println!(
                        "{:010.4?} thread: {:04}, count: {:09}, iops: {:9.3?}",
                        now,
                        i,
                        (index + 1),
                        (index + 1) as f64 / e,
                    );
                }
            }
        });
        handlers.push(h);
    }
    for h in handlers {
        total_length += h.join().unwrap();
    }
    let elapsed3 = SystemTime::now().duration_since(begin_at).unwrap() - elapsed2;
    println!(
        "total count: {}, total length: {}, elapsed: {:?}",
        slice.len(),
        total_length,
        elapsed3,
    );
}

// fn basic(path: String, max_keys: i64) {
//     let begin_at = SystemTime::now();
//     let entries = std::fs::read_dir(path).unwrap();
//     let elapsed1 = SystemTime::now().duration_since(begin_at).unwrap();
//     println!("read dir: {:?}", elapsed1);
//     let mut total_length = 0;
//     let begin_at = SystemTime::now();
//     let mut total_count = 0;
//     for (index, entry) in entries.enumerate() {
//         let b = SystemTime::now();
//         let entry: std::fs::DirEntry = entry.unwrap();
//         if entry.metadata().unwrap().is_file() {
//             let data = std::fs::read(entry.path()).unwrap();
//             total_length += data.len();
//         }
//         let e = SystemTime::now().duration_since(b).unwrap();
//         if index % 1000 == 0 {
//             println!("read file: {:?}", e);
//         }
//         if index % 10000 == 0 {
//             println!("count: {}", index);
//         }
//         total_count = index + 1;
//     }
//     let elapsed2 = SystemTime::now().duration_since(begin_at).unwrap();
//     println!(
//         "total count: {}, read files: {:?}, total length: {}",
//         total_count, elapsed2, total_length
//     );
// }
