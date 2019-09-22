pub trait Backend {}

#[derive(Debug)]
pub struct SimpleBackend {}

impl Backend for SimpleBackend {}
