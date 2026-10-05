

pub struct Env {
    argv: Vec<String>,
    is_main_thread: bool,
}

impl Env {
    pub fn new() -> Self {
        let argv: Vec<String> = std::env::args().collect();
        Self {
            argv,
            is_main_thread: true
        }
    }

    pub fn argv(&self) -> &Vec<String> {
        &self.argv
    }

    pub fn is_main_thread(&self) -> bool {
        self.is_main_thread
    }
}
