use crate::with_dot;
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

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
    need_del: Option<bool>,
    #[serde(rename = "ignoreExt")]
    ignore_ext: Option<Vec<String>>,
    #[serde(rename = "ignoreFiles")]
    ignore_files: Option<Vec<String>>,
    #[serde(rename = "ignoreDependencies")]
    ignore_dependencies: Option<Vec<String>>,
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
    let package_json_str = fs::read_to_string(package_json_path).unwrap();
    let pkg_json: ProjectDependencies = serde_json::from_str(package_json_str.as_str()).unwrap();
    let dependencies_info = ProjectDependencies {
        dependencies: pkg_json.dependencies,
        dev_dependencies: pkg_json.dev_dependencies,
    };
    dependencies_info.all_dependencies()
}

pub fn get_cli_arg_root_path() -> Option<String> {
    env::args().into_iter().nth(1)
}

pub fn load_user_config(project_root: &str) -> MinippConfig {
    let user_json_config_path = PathBuf::from(project_root).join("minipp.config.json");
    let user_json_config_str = fs::read_to_string(user_json_config_path).unwrap();
    let user_config: MinippConfig = serde_json::from_str(user_json_config_str.as_str()).unwrap();
    user_config
}

pub fn has_file_extension(file_path: &str) -> bool {
    let ext_option = Path::new(file_path).extension();
    if let Some(ext) = ext_option {
        SUPPORT_FILE_TYPES.contains(&ext.to_str().unwrap())
    } else {
        false
    }
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
}
