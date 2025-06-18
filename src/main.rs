use minipp_rs::processors::js_like::get_js_like_import_info;
use minipp_rs::processors::style_like::get_style_like_import_info;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[derive(Serialize)]
struct AllImport {
    imports: HashSet<String>,
    dependencies: HashSet<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let js_like_import = get_js_like_import_info();
    let style_like_import = get_style_like_import_info();
    let all_import = AllImport {
        dependencies: js_like_import.dependencies,
        imports: js_like_import
            .imports
            .into_iter()
            .chain(style_like_import.imports)
            .collect::<HashSet<_>>(),
    };
    let all_import_str = serde_json::to_string(&all_import)?;
    let mut file = File::create("minipp.report.json")?;
    file.write_all(all_import_str.as_bytes())?;
    println!("成功生成minipp.report.json文件！");
    Ok(())
}
