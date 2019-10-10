use clap::{App, Arg};

fn main() {
    let matches = App::new("dd")
        .version("1.0")
        .author("divinerapier")
        .about("file generator")
        .arg(
            Arg::with_name("dir_num")
                .required(true)
                .long("dir_num")
                .value_name("DIR_NUM")
                .help("Sets the number of directories")
                .takes_value(true)
                .validator(is_usize),
        )
        .arg(
            Arg::with_name("file_num")
                .required(true)
                .long("file_num")
                .value_name("FILE_NUM")
                .help("Sets the number of files")
                .takes_value(true)
                .validator(is_usize),
        )
        .arg(
            Arg::with_name("file_size")
                .required(true)
                .long("file_size")
                .value_name("FILE_SIZE")
                .help("Sets the size of file")
                .takes_value(true)
                .validator(is_usize),
        )
        .get_matches();
    let dir_num: usize = matches.value_of("dir_num").unwrap().parse().unwrap();
    let file_num: usize = matches.value_of("file_num").unwrap().parse().unwrap();
    let file_size: usize = matches.value_of("file_size").unwrap().parse().unwrap();
    dd(dir_num, file_num, file_size);
}

fn is_usize(arg: String) -> Result<(), String> {
    match arg.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
    }
}

fn dd(dir_num: usize, file_num: usize, file_size: usize) {
    fn generate_one(size: usize) -> Vec<u8> {
        let mut data = vec![0u8; size];
        srand::read(&mut data).unwrap();
        data
    }

    for dir_index in 0..dir_num {
        std::fs::create_dir_all(format!("./{}", dir_index)).unwrap();
        for file_index in 0..file_num {
            let path = format!("./{}/{}.dat", dir_index, file_index);
            let data = generate_one(file_size);
            std::fs::write(&path, &data).unwrap();
        }
        println!("generated {} files", (dir_index + 1) * dir_num);
    }
}
