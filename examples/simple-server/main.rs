use std::env;

fn main() {
    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or(env::var("LOG_LEVEL").unwrap_or(String::from("debug"))),
    )
    .init();
    log::set_max_level(log::LevelFilter::max());

    let mountpoint = env::var("MOUNT_POINT").unwrap_or(String::from("./mnt"));
    let rootpath = env::var("ROOT_PATH").unwrap_or(String::from("./root"));

    let fs = ossfs::Fuse::new(ossfs::SimpleBackend::new(rootpath));
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
