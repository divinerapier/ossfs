pub mod fs;

#[cfg(test)]
mod test {
    #[test]
    fn test_api_parameters() {
        env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        log::set_max_level(log::LevelFilter::max());
        std::fs::create_dir_all("./tmp");
        std::fs::create_dir_all("./mnt");
        // let fs = super::fs::Fuse::new(super::fs::SimpleBackend::new("./tmp"));
        let fs = super::fs::Fuse::new(super::fs::s3::S3Backend::new(
            "http://127.0.0.1:9001",
            "5577006791947779410",
            "admin",
            "password",
        ));
        let mountpoint = "./mnt";
        let options = ["-o", "rw", "-o", "fsname=cachefs"]
            .iter()
            .map(|o| o.as_ref())
            .collect::<Vec<&std::ffi::OsStr>>();
        fuse::mount(fs, &mountpoint, &options).unwrap();
    }
}
