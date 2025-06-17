use minipp_rs::processors::js_like::{get_js_like_import_info, ImportCollector};
use minipp_rs::processors::style_like::get_style_like_import_info;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let js_like_import = get_js_like_import_info();
    let style_like_import = get_style_like_import_info();
    let all_import = ImportCollector {
        dependencies: js_like_import.dependencies,
        imports: js_like_import
            .imports
            .into_iter() // 获取所有权
            .chain(style_like_import.imports) // 链接另一个集合
            .collect::<HashSet<_>>(),
    };
    let all_import_str = serde_json::to_string(&all_import)?;
    let mut file = File::create("minipp.report.json")?;
    file.write_all(all_import_str.as_bytes())?;
    println!("成功生成minipp.report.json文件！");
    Ok(())
}
