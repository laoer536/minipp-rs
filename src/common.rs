use crate::with_dot;
use ignore::gitignore::GitignoreBuilder;
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Default, Debug, Deserialize)]
pub struct ProjectDependencies {
    #[serde(rename = "dependencies")]
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

impl ProjectDependencies {
    pub fn all_dependencies(&self) -> HashSet<String> {
        let mut set: HashSet<String> = HashSet::new();
        for map_option in [&self.dependencies, &self.dev_dependencies] {
            if let Some(map) = map_option {
                for (k, _) in map {
                    set.insert(k.to_string());
                }
            }
        }
        set
    }
}

#[derive(Default, Debug, Deserialize, PartialEq)]
pub struct MinippConfig {
    #[serde(rename = "needDel")]
    pub need_del: Option<bool>,
    #[serde(rename = "ignoreExt")]
    pub ignore_ext: Option<Vec<String>>,
    #[serde(rename = "ignoreFiles")]
    pub ignore_files: Option<Vec<String>>,
    #[serde(rename = "ignoreDependencies")]
    pub ignore_dependencies: Option<Vec<String>>,
}

pub const BACK_UP_FOLDER: &str = "minipp-delete-files";

pub const SUPPORT_FILE_TYPES: [&str; 18] = [
    "ts", "tsx", "less", "scss", "css", "png", "jpg", "jpeg", "gif", "svg", "mp3", "mp4", "wav",
    "woff", "woff2", "ttf", "eot", "json",
];

// This is just for the convenience of copying, of course, it is completely possible to write them one by one instead of using macro_rules.
pub const SUPPORT_FILE_TYPES_WITH_DOT: [&str; 18] = with_dot![
    "ts", "tsx", "less", "scss", "css", "png", "jpg", "jpeg", "gif", "svg", "mp3", "mp4", "wav",
    "woff", "woff2", "ttf", "eot", "json"
];

pub fn get_project_dependencies(project_root: &str) -> HashSet<String> {
    let package_json_path = PathBuf::from(project_root).join("package.json");
    let package_json_str =
        fs::read_to_string(package_json_path).expect("Unable to read package.json");
    let pkg_json: ProjectDependencies = serde_json::from_str(package_json_str.as_str()).unwrap();
    let dependencies_info = ProjectDependencies {
        dependencies: pkg_json.dependencies,
        dev_dependencies: pkg_json.dev_dependencies,
    };
    dependencies_info.all_dependencies()
}

pub fn get_project_root_path() -> Result<String, io::Error> {
    // 优先取命令行参数
    if let Some(arg1) = env::args().nth(1) {
        Ok(arg1)
    } else {
        // 没有参数则用当前目录
        env::current_dir().and_then(|p| {
            p.to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))
                .map(|p| p.to_string())
        })
    }
}

pub fn load_user_config(project_root: &str) -> MinippConfig {
    let user_json_config_path = PathBuf::from(project_root).join("minipp.config.json");
    if let Ok(config_json) = fs::read_to_string(user_json_config_path) {
        let user_config: MinippConfig = serde_json::from_str(config_json.as_str()).unwrap();
        user_config
    } else {
        MinippConfig::default()
    }
}

pub fn has_file_extension(file_path: &str) -> bool {
    let ext_option = Path::new(file_path).extension();
    if let Some(ext) = ext_option {
        SUPPORT_FILE_TYPES.contains(&ext.to_str().unwrap())
    } else {
        false
    }
}

pub fn multi_pattern_filter(files: &[String], patterns: &[String]) -> Vec<String> {
    // 创建忽略规则构建器（当前目录为根）
    let mut builder = GitignoreBuilder::new("");
    // 添加所有 pattern
    for pat in patterns {
        builder.add_line(None, pat).unwrap();
    }
    let gitignore = builder.build().unwrap();

    files
        .iter()
        .filter(|file| {
            // 用 matched_path_or_any_parents 来递归判断父目录是否被 ignore
            !gitignore
                .matched_path_or_any_parents(Path::new(file), false)
                .is_ignore()
        })
        .cloned()
        .collect()
}

pub fn is_path_ignored(file: &str, patterns: &[String]) -> bool {
    let mut builder = GitignoreBuilder::new("");
    for pattern in patterns {
        builder.add_line(None, pattern).unwrap();
    }
    let gitignore = builder.build().unwrap();

    gitignore
        .matched_path_or_any_parents(Path::new(file), false)
        .is_ignore()
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_ROOT_PATH: &str = "/Users/neo/Desktop/neo/github/minip";

    #[test]
    fn file_types_should_dot() {
        assert_eq!(
            SUPPORT_FILE_TYPES_WITH_DOT,
            [
                ".ts", ".tsx", ".less", ".scss", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg",
                ".mp3", ".mp4", ".wav", ".woff", ".woff2", ".ttf", ".eot", ".json",
            ]
        )
    }

    #[test]
    fn test_get_project_dependencies() {
        let project_root = USER_ROOT_PATH;
        let project_dependencies = get_project_dependencies(project_root);
        let should_result = HashSet::from([
            "@swc/cli",
            "@types/node",
            "@typescript-eslint/eslint-plugin",
            "@typescript-eslint/parser",
            "@vitest/coverage-v8",
            "changelogen",
            "eslint",
            "glob",
            "ignore",
            "prettier",
            "tsx",
            "typescript",
            "unbuild",
            "vitest",
            "yocto-spinner",
            "@swc/core",
        ])
        .iter()
        .map(|dependence| dependence.to_string())
        .collect();
        assert_eq!(project_dependencies, should_result);
    }

    #[test]
    fn test_load_user_config() {
        let project_root = USER_ROOT_PATH;
        assert_eq!(
            load_user_config(project_root),
            MinippConfig {
                need_del: Some(false),
                ignore_ext: None,
                ignore_files: Some(vec!["src/index.ts".to_string(), "src/core/**".to_string()]),
                ignore_dependencies: Some(vec!["@types*".to_string(), "eslint".to_string()]),
            }
        );
    }

    #[test]
    fn test_has_file_extension() {
        assert_eq!(has_file_extension("src/main.rs"), false);
        assert_eq!(has_file_extension("src/main.ts"), true)
    }

    #[test]
    fn test_multi_pattern_filter_basic() {
        let files = vec![
            "foo.txt".to_string(),
            "bar.log".to_string(),
            "baz/test.txt".to_string(),
            "baz/foo.log".to_string(),
            "qux.rs".to_string(),
        ];
        let patterns = vec!["*.log".to_string(), "baz/".to_string()];
        let filtered = multi_pattern_filter(&files, &patterns);
        assert_eq!(filtered, vec!["foo.txt".to_string(), "qux.rs".to_string()]);
    }

    #[test]
    fn test_multi_pattern_filter_no_patterns() {
        let files = vec!["foo.txt".to_string(), "bar.log".to_string()];
        let patterns: Vec<String> = vec![];
        let filtered = multi_pattern_filter(&files, &patterns);
        assert_eq!(filtered, files);
    }

    #[test]
    fn test_multi_pattern_filter_all_ignored() {
        let files = vec![
            "foo.txt".to_string(),
            "bar.log".to_string(),
            "baz/test.txt".to_string(),
        ];
        let patterns = vec!["*".to_string()];
        let filtered = multi_pattern_filter(&files, &patterns);
        assert_eq!(filtered, Vec::<String>::new());
    }

    #[test]
    fn test_multi_pattern_filter_negation() {
        let files = vec![
            "foo.txt".to_string(),
            "bar.log".to_string(),
            "baz/test.txt".to_string(),
        ];
        let patterns = vec!["*.log".to_string(), "!bar.log".to_string()];
        let filtered = multi_pattern_filter(&files, &patterns);
        assert_eq!(
            filtered,
            vec![
                "foo.txt".to_string(),
                "bar.log".to_string(),
                "baz/test.txt".to_string()
            ]
        );
    }

    #[test]
    fn test_is_path_ignored() {
        let patterns = ["*.rs", "target/"].map(|i| i.to_string());
        assert!(is_path_ignored("main.rs", &patterns));
        assert!(is_path_ignored("target/foo.o", &patterns));
        assert!(!is_path_ignored("foo/bar.txt", &patterns));
    }
}
