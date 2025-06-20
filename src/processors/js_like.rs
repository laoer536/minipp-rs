use crate::common::{get_project_root_path, has_file_extension};
use glob::glob;
use path_clean::clean;
use std::collections::HashSet;
use std::path::Path;
use std::{fs, io};
use swc_common::errors::{ColorConfig, Handler};
use swc_common::input::StringInput;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{CallExpr, Callee, EsVersion, Expr, ImportDecl, JSXAttr, JSXExpr, Module};
use swc_ecma_ast::{JSXAttrValue, Lit};
use swc_ecma_parser::{Lexer, Parser, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Default, Debug)]
pub struct ImportCollector {
    pub imports: HashSet<String>,
    pub dependencies: HashSet<String>,
    pub current_file_path: String,
    pub all_files: HashSet<String>,
}

impl ImportCollector {
    fn jsx_attr_insert(&mut self, path: &str) {
        if has_file_extension(path) {
            let real_path = path_to_real_path(&self.current_file_path, path);
            if let Ok(s) = real_path {
                self.imports.insert(s);
            }
        }
    }

    fn common_insert(&mut self, path: &str) {
        let real_path = path_to_real_path(self.current_file_path.as_str(), path);
        if let Ok(s) = real_path {
            if s.contains("node_modules") {
                return;
            }
            if s.starts_with("src/") {
                self.imports.insert(s);
            } else {
                self.dependencies.insert(s);
            }
        }
    }
}

impl Visit for ImportCollector {
    fn visit_call_expr(&mut self, node: &CallExpr) {
        if let Callee::Import(_i) = &node.callee {
            for arg in &node.args {
                let expr = &*arg.expr;
                if let Expr::Lit(Lit::Str(s)) = expr {
                    self.common_insert(&s.value);
                }
            }
        }
        node.visit_children_with(self)
    }
    fn visit_import_decl(&mut self, import_node: &ImportDecl) {
        self.common_insert(&import_node.src.value);
        import_node.visit_children_with(self);
    }

    fn visit_jsx_attr(&mut self, node: &JSXAttr) {
        if let Some(value) = &node.value {
            match value {
                JSXAttrValue::Lit(Lit::Str(s)) => self.jsx_attr_insert(&s.value),
                JSXAttrValue::JSXExprContainer(jsx_expr_container) => {
                    if let JSXExpr::Expr(expr) = &jsx_expr_container.expr {
                        match &**expr {
                            Expr::Lit(Lit::Str(s)) => self.jsx_attr_insert(&s.value),
                            Expr::Tpl(tpl) => {
                                for quasi in &tpl.quasis {
                                    self.jsx_attr_insert(&quasi.raw)
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

pub fn try_to_find_files_without_a_suffix(
    relative_path_for_project: &str,
    all_files: &HashSet<String>,
) -> String {
    if has_file_extension(relative_path_for_project) {
        return relative_path_for_project.into();
    }
    let candidates = [
        format!("{}.ts", relative_path_for_project),
        format!("{}.tsx", relative_path_for_project),
        format!("{}/index.ts", relative_path_for_project),
        format!("{}/index.tsx", relative_path_for_project),
        format!("{}.d.ts", relative_path_for_project),
        format!("{}/index.d.ts", relative_path_for_project),
    ];

    for candidate in &candidates {
        if all_files.contains(candidate) {
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
                        match fs::read_to_string(&path) {
                            Ok(code) => {
                                let module = parse_ts_or_tsx(&code);
                                match path.to_str() {
                                    Some(path) => {
                                        import_collector.current_file_path = path.to_string();
                                        import_collector.all_files.insert(path.to_string());
                                    }
                                    None => continue,
                                };
                                // path.to_str().unwrap().to_string();
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

// 用于产生src/开头的路径、文件或者依赖 后续会将文件夹路径统一还原为文件路径(如果有的话)
fn path_to_real_path(current_file_path: &str, import_path: &str) -> Result<String, io::Error> {
    if import_path.starts_with("@/") {
        return Ok(current_file_path.replace("@/", "src/"));
    }

    if import_path.starts_with("..") || import_path.starts_with(".") {
        // 创建基础路径并获取父目录
        let base_path = Path::new(current_file_path);
        let parent = base_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Current path has no parent directory",
            )
        })?;

        // 构建完整路径并解析为规范路径
        let full_path = parent.join(import_path);
        let canonical_path = clean(&full_path);

        // 转换为字符串（处理无效Unicode）
        canonical_path.into_os_string().into_string().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Path contains invalid UTF-8 characters",
            )
        })
    } else if import_path.contains("src") {
        let root = get_project_root_path()?;
        let base = Path::new(&root);
        let abs = Path::new(import_path);
        Ok(abs
            .strip_prefix(base)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .to_str()
            .unwrap()
            .to_string())
    } else {
        Ok(import_path.to_string())
    }
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
        import_collector.current_file_path = String::from("src/core/cli/index.ts");
        module.visit_with(&mut import_collector);
        let should_import_res =
            HashSet::from(["src/core/common", "src/core/visitor"].map(String::from));
        let should_dependence_res =
            HashSet::from(["fs", "util", "@swc/core", "path", "glob"].map(String::from));
        assert_eq!(import_collector.imports, should_import_res);
        assert_eq!(import_collector.dependencies, should_dependence_res);
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
        import_collector.current_file_path = String::from("src/core/cli/index.ts");
        module.visit_with(&mut import_collector);
        let should_res =
            HashSet::from(["src/core/cli/Type19", "src/core/cli/Type20"].map(String::from));
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
        import_collector.current_file_path = String::from("src/index.tsx");
        module.visit_with(&mut import_collector);
        let should_res = HashSet::from(["src/assets/b.jpg", "src/assets/a.jpg"].map(String::from));
        assert_eq!(import_collector.imports, should_res);
    }

    #[test]
    fn test_path_to_real_path() {
        let current_path = "src/components/CourseForm/index.tsx";
        let import_path = "../ColorPicker";
        let should_path = "src/components/ColorPicker";
        assert_eq!(
            path_to_real_path(current_path, import_path).unwrap(),
            should_path.to_string()
        );
    }
}
