use crate::common::has_file_extension;
use glob::glob;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs};
use swc_common::errors::{ColorConfig, Handler};
use swc_common::input::StringInput;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{CallExpr, EsVersion, ImportDecl, JSXAttr, Module};
use swc_ecma_parser::{Lexer, Parser, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Default)]
struct ImportCollector {
    imports: HashSet<String>,
}

impl Visit for ImportCollector {
    fn visit_call_expr(&mut self, node: &CallExpr) {
        //TODO
        node.visit_children_with(self)
    }
    fn visit_import_decl(&mut self, import_node: &ImportDecl) {
        let import = import_node.src.value.to_string();
        self.imports.insert(import);
        import_node.visit_children_with(self);
    }

    fn visit_jsx_attr(&mut self, node: &JSXAttr) {
        //TODO
        node.visit_children_with(self)
    }
}

fn parse_ts_or_tsx(code: String) -> Module {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm.new_source_file(FileName::Custom("virtual.tsx".into()).into(), code);
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: false,
            ..Default::default()
        }),
        // EsVersion defaults to es5
        EsVersion::EsNext,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");
    module
}

fn try_to_find_files_without_a_suffix(relative_path_for_project: String) -> String {
    let candidates = [
        format!("{}.ts", relative_path_for_project),
        format!("{}.tsx", relative_path_for_project),
        format!("{}/index.ts", relative_path_for_project),
        format!("{}/index.tsx", relative_path_for_project),
        format!("{}.d.ts", relative_path_for_project),
        format!("{}/index.d.ts", relative_path_for_project),
    ];

    for candidate in &candidates {
        if Path::new(&candidate).exists() {
            return candidate.to_string();
        }
    }
    format!(
        "{}(Unknown file type, the file does not exist in the scan directory, or is not a TSX, TS or .d.ts file)",
        relative_path_for_project
    )
}

pub fn get_js_like_import_info() {
    let mut import_collector = ImportCollector::default();
    for entry in glob("src/**/*.{ts,tsx}").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    let code = fs::read_to_string(path).unwrap();
                    let module = parse_ts_or_tsx(code);
                    module.visit_with(&mut import_collector);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

fn path_to_real_path(current_file_path: String, import_path: String) -> String {
    if import_path.starts_with("@/") {
        let relative_path_for_project = import_path.replace("@/", "src/");
        return if has_file_extension(&relative_path_for_project) {
            relative_path_for_project
        } else {
            try_to_find_files_without_a_suffix(relative_path_for_project)
        };
    }

    if import_path.starts_with("..") || import_path.starts_with(".") {
        let get_path = |base: &str, path: &str| -> String {
            PathBuf::from(base)
                .join(path)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        };
        let absolute_path = get_path(&current_file_path, &import_path);
        //TODO fix rust中并没有node中的path.relative，需要手动实现
        let relative_path_for_project = get_path(
            env::current_dir().unwrap().to_str().unwrap(),
            &absolute_path,
        );
        return if has_file_extension(&relative_path_for_project) {
            relative_path_for_project
        } else {
            try_to_find_files_without_a_suffix(relative_path_for_project)
        };
    }
    import_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_ecma_ast::ModuleItem;

    #[test]
    fn test_parse_ts_code() {
        let code = "const a: number = 123;";
        let module: Module = parse_ts_or_tsx(code.to_string());
        assert!(
            !module.body.is_empty(),
            "TS parse result should not be empty"
        );
    }
    #[test]
    fn test_parse_tsx_code() {
        let code = r#"
            import React from 'react';
            export const App = () => <div>Hello TSX</div>;
        "#;
        let module: Module = parse_ts_or_tsx(code.to_string());
        assert!(
            !module.body.is_empty(),
            "TSX parse result should not be empty"
        );

        // 检查是否有导出声明
        let has_export = module
            .body
            .iter()
            .any(|item| matches!(item, ModuleItem::ModuleDecl(_)));
        assert!(has_export, "Should have at least one export in TSX code");
    }
}
