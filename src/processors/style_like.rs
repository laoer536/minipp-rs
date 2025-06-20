use glob::glob;
use path_clean::{clean, PathClean};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::{fs, io};

#[derive(Default, Debug)]
pub struct StyleImportCollector {
    pub imports: HashSet<String>,
    pub current_file_path: String,
}

impl StyleImportCollector {
    pub fn insert_from_code(&mut self, code: &str) {
        let imports = get_extract_style_imports(code).into_iter().map(|i| {
            let real_path = path_to_real_path(&self.current_file_path, &i);
            if let Ok(s) = real_path {
                s
            } else {
                "unknown".to_string()
            }
        });
        self.imports.extend(imports);
    }
}
fn get_extract_style_imports(code: &str) -> Vec<String> {
    // 正则表达式和 TS 版本一致
    let regex = Regex::new(
        r#"@import\s+(?:url\()?['"]?([^'")]+)['"]?\)?|url\(\s*['"]?([^'")]+)['"]?\s*\)"#,
    )
    .unwrap();
    let mut result = Vec::new();
    for cap in regex.captures_iter(code) {
        // 获取匹配到的路径
        let raw_path = cap
            .get(1)
            .and_then(|m| Some(m.as_str()))
            .or_else(|| cap.get(2).map(|m| m.as_str()));
        let Some(raw_path) = raw_path else { continue };
        // 跳过包含 { $ # 的动态路径
        if raw_path.contains('{') || raw_path.contains('$') || raw_path.contains('#') {
            continue;
        }
        // 去掉 ? # 后面的部分并去除空白
        let path = raw_path
            .split(|c| c == '?' || c == '#')
            .next()
            .unwrap()
            .trim();
        // 排除 http(s)://、//、/ 开头的绝对路径
        if !Regex::new(r#"^([a-z]+:)?//"#).unwrap().is_match(path) && !path.starts_with('/') {
            result.push(raw_path.to_string());
        }
    }
    result
}

pub fn get_style_like_import_info() -> StyleImportCollector {
    let mut style_import_collector = StyleImportCollector::default();
    let patterns = ["src/**/*.css", "src/**/*.less", "src/**/*.scss"];
    for pattern in patterns {
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        let code = fs::read_to_string(&path).unwrap();
                        style_import_collector.current_file_path =
                            path.to_str().unwrap().to_string();
                        style_import_collector.insert_from_code(&code);
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
    style_import_collector
}

fn path_to_real_path(current_file_path: &str, import_path: &str) -> Result<String, io::Error> {
    if import_path.starts_with("@/") {
        return Ok(current_file_path.replace("@/", "src/"));
    }
    if import_path.starts_with("~") {
        return if import_path.ends_with("~@") {
            Ok(import_path.replace("~@", "src/"))
        } else {
            Ok(format!("{}(Non-project code file)", import_path))
        };
    }

    let parent = Path::new(current_file_path)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;
    let should_path = clean(parent.join(import_path));
    should_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_get_style_imports() {
        let style_code = r#"
        /* 1. 使用 url() */
body {
  background-image: url('images/bg.jpg');
}

.icon {
  background: url("icons/icon.svg") no-repeat;
}

@import "reset.css";
@import url("theme.css");
@import url('https://fonts.googleapis.com/css?family=Roboto');

/* 3. 字体引入 */
@font-face {
  font-family: 'MyFont';
  src: url('fonts/myfont.woff2') format('woff2');
}
        "#;
        let should_res: Vec<_> = vec![
            "images/bg.jpg",
            "icons/icon.svg",
            "reset.css",
            "fonts/myfont.woff2",
            "theme.css",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            get_extract_style_imports(style_code)
                .iter()
                .collect::<HashSet<_>>(),
            should_res.iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn test_path_to_real_path() {
        assert_eq!(
            path_to_real_path("src/foo/bar.less", "./common.less").unwrap(),
            "src/foo/common.less".to_string()
        );
        assert_eq!(
            path_to_real_path("src/foo/bar.less", "common.less").unwrap(),
            "src/foo/common.less".to_string()
        );
    }
}
