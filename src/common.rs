use crate::with_dot;
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

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

#[derive(Default, Debug)]
pub struct MinippConfig {
    need_del: bool,
    ignore_ext: Vec<String>,
    ignore_files: Vec<String>,
    ignore_dependencies: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::{get_project_dependencies, SUPPORT_FILE_TYPES_WITH_DOT};
    use std::collections::HashSet;
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
        let project_root = "/Users/neo/Desktop/neo/github/minip";
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
}
