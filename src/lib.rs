mod error;
mod ossfs_impl;

pub use ossfs_impl::backend::{s3::S3Backend, simple::SimpleBackend, Backend};
pub use ossfs_impl::Fuse;

#[cfg(test)]
mod test {
    #[test]
    fn test_api_parameters() {
        env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();
        log::set_max_level(log::LevelFilter::max());
        std::fs::create_dir_all("./tmp");
        std::fs::create_dir_all("./mnt");
        // let fs = super::fs::Fuse::new(super::fs::SimpleBackend::new("./tmp"));
        let fs = super::Fuse::new(super::S3Backend::new(
            "http://172.21.20.134:9001",
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
