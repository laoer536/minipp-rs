use crate::common::has_file_extension;
use glob::glob;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use swc_common::errors::{ColorConfig, Handler};
use swc_common::input::StringInput;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{CallExpr, Callee, EsVersion, Expr, ImportDecl, JSXAttr, JSXExpr, Module};
use swc_ecma_ast::{JSXAttrValue, Lit};
use swc_ecma_parser::{Lexer, Parser, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Default, Debug, Serialize)]
pub struct ImportCollector {
    pub imports: HashSet<String>,
    pub dependencies: HashSet<String>,
}

impl ImportCollector {
    fn deal_jsx_attr_insert(&mut self, path: &str) {
        if has_file_extension(path) {
            self.imports.insert(path.to_string());
        }
    }
}

impl Visit for ImportCollector {
    fn visit_call_expr(&mut self, node: &CallExpr) {
        if let Callee::Import(_i) = &node.callee {
            for arg in &node.args {
                let expr = &*arg.expr;
                if let Expr::Lit(Lit::Str(s)) = expr {
                    self.imports.insert(s.value.to_string());
                }
            }
        }
        node.visit_children_with(self)
    }
    fn visit_import_decl(&mut self, import_node: &ImportDecl) {
        let import = import_node.src.value.to_string();
        self.imports.insert(import);
        import_node.visit_children_with(self);
    }

    fn visit_jsx_attr(&mut self, node: &JSXAttr) {
        if let Some(value) = &node.value {
            match value {
                JSXAttrValue::Lit(Lit::Str(s)) => {
                    self.deal_jsx_attr_insert(s.value.as_str());
                }
                JSXAttrValue::JSXExprContainer(jsx_expr_container) => {
                    if let JSXExpr::Expr(expr) = &jsx_expr_container.expr {
                        match &**expr {
                            Expr::Lit(Lit::Str(s)) => {
                                self.deal_jsx_attr_insert(s.value.as_str());
                            }
                            Expr::Tpl(tpl) => {
                                for quasi in &tpl.quasis {
                                    self.deal_jsx_attr_insert(quasi.raw.as_str())
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        node.visit_children_with(self)
    }
}

fn parse_ts_or_tsx(code: &str) -> Module {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm.new_source_file(
        FileName::Custom("virtual.tsx".into()).into(),
        code.to_string(),
    );
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

fn try_to_find_files_without_a_suffix(relative_path_for_project: &str) -> String {
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

pub fn get_js_like_import_info() -> ImportCollector {
    let mut import_collector = ImportCollector::default();
    let patterns = ["src/**/*.ts", "src/**/*.tsx"];
    for pattern in patterns {
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        match fs::read_to_string(path) {
                            Ok(code) => {
                                let module = parse_ts_or_tsx(code.as_str());
                                module.visit_with(&mut import_collector);
                            }
                            Err(e) => {
                                println!("{:?}", e);
                                continue;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            }
        }
    }
    import_collector
}

fn path_to_real_path(current_file_path: String, import_path: String) -> Result<String, io::Error> {
    if import_path.starts_with("@/") {
        let relative_path_for_project = import_path.replace("@/", "src/");
        return if has_file_extension(&relative_path_for_project) {
            Ok(relative_path_for_project)
        } else {
            Ok(try_to_find_files_without_a_suffix(
                &relative_path_for_project,
            ))
        };
    }

    if import_path.starts_with("..") || import_path.starts_with(".") {
        let get_path = |base: &str, path: &str| -> Result<String, io::Error> {
            let buf = PathBuf::from(base).join(path).canonicalize()?;
            Ok(buf
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 path"))?
                .to_string())
        };
        let absolute_path = get_path(&current_file_path, &import_path)?;
        let current_dir = env::current_dir()?;
        let relative_path_for_project = get_path(
            current_dir
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 path"))?,
            &absolute_path,
        )?;
        return if has_file_extension(&relative_path_for_project) {
            Ok(relative_path_for_project)
        } else {
            Ok(try_to_find_files_without_a_suffix(
                &relative_path_for_project,
            ))
        };
    }
    Ok(import_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_ecma_ast::ModuleItem;

    #[test]
    fn test_parse_ts_code() {
        let code = "const a: number = 123;";
        let module: Module = parse_ts_or_tsx(code);
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
        let module: Module = parse_ts_or_tsx(code);
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

    #[test]
    fn should_collect_import() {
        let code = r#"
import {
  type ImportDeclaration,
  parse,
  type TsType,
  type JSXAttribute,
  type CallExpression,
  type Expression,
} from '@swc/core'
import Visitor from '../visitor'
import { glob } from 'glob'
import path from 'path'
import fs from 'fs'
import { styleText } from 'util'
import { hasFileExtension } from '../common'
        "#;
        let module = parse_ts_or_tsx(code);
        let mut import_collector = ImportCollector::default();
        module.visit_with(&mut import_collector);
        let should_res = HashSet::from(
            [
                "../common",
                "fs",
                "util",
                "@swc/core",
                "path",
                "../common",
                "glob",
                "../visitor",
            ]
            .map(String::from),
        );
        assert_eq!(import_collector.imports, should_res);
    }

    #[test]
    fn should_collect_dy_import() {
        let code = r#"
        const index: Map<number, React.ComponentType<ContentFormStep3Interface>> = new Map()
  .set(
    20,
    React.lazy(() => import('./Type20'))
  )
  .set(
    2,
    React.lazy(() => import('./Type19'))
  );
        "#;
        let module = parse_ts_or_tsx(code);
        let mut import_collector = ImportCollector::default();
        module.visit_with(&mut import_collector);
        let should_res = HashSet::from(["./Type19", "./Type20"].map(String::from));
        assert_eq!(import_collector.imports, should_res);
    }

    #[test]
    fn should_collect_assets_import() {
        let code = r#"
export default function DomStringSrcTest() {
  return (
    <div>
      {/* 直接字符串路径 */}
      <img src={"./assets/b.jpg"} alt="花括号字符串字面量" />
      <img src="./assets/a.jpg" alt="花括号字符串字面量" />
    </div>
  );
}
        "#;
        let module = parse_ts_or_tsx(code);
        let mut import_collector = ImportCollector::default();
        module.visit_with(&mut import_collector);
        let should_res = HashSet::from(["./assets/b.jpg", "./assets/a.jpg"].map(String::from));
        assert_eq!(import_collector.imports, should_res);
    }
}
