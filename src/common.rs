use crate::with_dot;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct ProjectDependencies {
    dependencies: HashMap<String, String>,
    dev_dependencies: HashMap<String, String>,
}

#[derive(Default, Debug)]
pub struct MinippConfig {
    need_del: bool,
    ignore_ext: Vec<String>,
    ignore_files: Vec<String>,
    ignore_dependencies: Vec<String>,
}

pub const BACK_UP_FOLDER: &str = "minipp-delete-files";

pub const SUPPORT_FILE_TYPES: [&str; 18] = [
    "css", "tsx", "less", "scss", "css", "png", "jpg", "jpeg", "gif", "svg", "mp3", "mp4", "wav",
    "woff", "woff2", "ttf", "eot", "json",
];

// This is just for the convenience of copying, of course, it is completely possible to write them one by one instead of using macro_rules.
pub const SUPPORT_FILE_TYPES_WITH_DOT: [&str; 18] = with_dot![
    "css", "tsx", "less", "scss", "css", "png", "jpg", "jpeg", "gif", "svg", "mp3", "mp4", "wav",
    "woff", "woff2", "ttf", "eot", "json"
];

#[cfg(test)]
mod tests {
    use super::SUPPORT_FILE_TYPES_WITH_DOT;
    #[test]
    fn file_types_should_dot() {
        assert_eq!(
            SUPPORT_FILE_TYPES_WITH_DOT,
            [
                ".css", ".tsx", ".less", ".scss", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg",
                ".mp3", ".mp4", ".wav", ".woff", ".woff2", ".ttf", ".eot", ".json",
            ]
        )
    }
}
