use clap::*;

fn main() {
    let matches = App::new("md5checker")
        .version("1.0")
        .author("divinerapier")
        .about("check files' md5")
        .arg(
            Arg::with_name("source")
                .required(true)
                .short("s")
                .long("source")
                .value_name("SOURCE")
                .help("source of mountpoint")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target")
                .required(true)
                .short("t")
                .long("target")
                .value_name("RECURSIVE")
                .help("target of mountpoint")
                .takes_value(true),
        )
        .get_matches();
    let source = matches.value_of("source").unwrap();
    let target = matches.value_of("target").unwrap();
    check(source.to_owned(), target.to_owned(), 32);
}

fn check(source: String, target: String, concurrency: usize) {
    let begin_at = std::time::SystemTime::now();
    // let entries = std::fs::read_dir(path).unwrap();
    let elapsed1 = std::time::SystemTime::now()
        .duration_since(begin_at)
        .unwrap();
    println!("read dir: {:?}", elapsed1);
    let begin_at = std::time::SystemTime::now();
    let wk = walkdir::WalkDir::new(source.clone()).into_iter();
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
    let target = std::sync::Arc::new(target);
    let source = std::sync::Arc::new(source);
    let mut handlers = vec![];
    for i in 0..concurrency {
        let slice = slice.clone();
        let target = target.clone();
        let source = source.clone();
        let h = std::thread::spawn(move || {
            let mut total_count = 0;
            let mut local_count = 0;
            for index in (i..slice.len()).step_by(concurrency) {
                let source_data = std::fs::read(&slice[index]).unwrap();
                let path: &str = &slice[index];
                let source: &str = &source;
                let target_path = path.replace(source, &target);
                // FIXME: if target's filesystem is ossfs, this will make server panics
                // workaround: run `tree` command at first
                let target_data = std::fs::read(&target_path).unwrap();

                let source_digest: md5::Digest = md5::compute(&source_data);
                let target_digest: md5::Digest = md5::compute(&target_data);

                if total_count < 10 {
                    println!(
                        "source: [{} {:x}], target: [{} {:x}]",
                        &slice[index], source_digest, target_path, target_digest
                    );
                }

                if !source_digest.eq(&target_digest) {
                    println!(
                        "source: [{} {:x}], target: [{} {:x}]",
                        &slice[index], source_digest, target_path, target_digest
                    );
                }

                total_count += source_data.len();
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
