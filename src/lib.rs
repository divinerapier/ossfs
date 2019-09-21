// #[macro_use]
// extern crate function_name;

pub mod fs;

#[cfg(test)]
mod test {
    #[test]
    fn test_api_parameters() {
        env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        log::set_max_level(log::LevelFilter::max());

        let fs = super::fs::cachefs::CacheFs::new();
        let mountpoint = "./mnt";
        let options = ["-o", "ro", "-o", "fsname=cachefs"]
            .iter()
            .map(|o| o.as_ref())
            .collect::<Vec<&std::ffi::OsStr>>();
        fuse::mount(fs, &mountpoint, &options).unwrap();
    }
}
