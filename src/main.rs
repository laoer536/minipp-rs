use minipp_rs::common::{get_project_root_path, load_user_config};
use minipp_rs::processors::js_like::{get_js_like_import_info, try_to_find_files_without_a_suffix};
use minipp_rs::processors::style_like::get_style_like_import_info;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Serialize)]
struct AllImport {
    imports: HashSet<String>,
    dependencies: HashSet<String>,
    unused_imports: HashSet<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let project_root = get_project_root_path()?;
    let minipp_config = load_user_config(&project_root);
    println!("{:?}", minipp_config);

    let (js_import, style_import) = rayon::join(
        || get_js_like_import_info(),
        || get_style_like_import_info(),
    );

    let all_imports: HashSet<_> = js_import
        .imports
        .iter()
        .map(|imp| try_to_find_files_without_a_suffix(imp, &js_import.all_files))
        .chain(style_import.imports)
        .collect();

    let unused_imports: HashSet<_> = js_import
        .all_files
        .difference(&all_imports)
        .cloned()
        .collect();

    let all_import = AllImport {
        dependencies: js_import.dependencies,
        imports: all_imports,
        unused_imports,
    };

    let report = serde_json::to_string_pretty(&all_import)?;
    File::create("minipp.report.json")?.write_all(report.as_bytes())?;

    println!("成功生成 minipp.report.json 文件！");
    println!("Time elapsed: {:?}", start.elapsed());
    Ok(())
}
