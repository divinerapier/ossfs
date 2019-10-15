use clap::{App, Arg};
use std::env;
fn main() {
    let matches = App::new("simple-server")
        .version("1.0")
        .author("divinerapier")
        .about("a simple ossfs server")
        .arg(
            Arg::with_name("backend")
                .required(false)
                .short("b")
                .long("backend")
                .value_name("BACKEND")
                .help("Sets the custom backend")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache")
                .required(false)
                .short("c")
                .long("cache")
                .help("Enable data cache")
                .takes_value(false),
        )
        .get_matches();
    let backend: String = if let Some(backend) = matches.value_of("backend") {
        backend.to_owned()
    } else {
        "".to_owned()
    };
    let enable_cache: bool = matches.is_present("cache");

    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or(env::var("LOG_LEVEL").unwrap_or(String::from("debug"))),
    )
    .init();
    log::set_max_level(log::LevelFilter::max());

    let mountpoint = env::var("MOUNT_POINT").unwrap_or(String::from("./mnt"));
    let rootpath = env::var("ROOT_PATH").unwrap_or(String::from("./root"));

    let fs = ossfs::Fuse::new(ossfs::SimpleBackend::new(rootpath), enable_cache);
    // let fs = super::Fuse::new(super::S3Backend::new(
    //     "http://172.21.20.134:9001",
    //     "5577006791947779410",
    //     "admin",
    //     "password",
    // ));
    let options = ["-o", "rw", "-o", "fsname=ossfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&std::ffi::OsStr>>();
    fuse::mount(fs, &mountpoint, &options).unwrap();
}
