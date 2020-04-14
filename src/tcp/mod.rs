pub struct Server<'a> {
    pub addr: &'a str,
}

impl<'a> Server<'a> {
    fn new() -> Server<'a> {
        Server {
            addr: "0.0.0.0:3057",
        }
    }
    fn run() {}
}
