use std::env;


#[derive(Debug, Clone)]
pub struct Config {
    pub directory: Option<String>,
}

impl Config {
    pub fn parse_config() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut directory = None;
    
        for i in 1..args.len() {
            if args[i] == "--directory" {
                if let Some(dir) = args.get(i + 1) {
                    directory = Some(dir.clone());
                }
        }
    }
        Self {directory: directory}
    }
}