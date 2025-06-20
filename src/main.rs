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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now(); // 记录开始时间
    let cli_arg_root_path =
        get_project_root_path().expect("Failed to get the root path of the project.");
    let minipp_config = load_user_config(&cli_arg_root_path);
    println!("{:?}", minipp_config);
    let js_like_import = get_js_like_import_info();
    let style_like_import = get_style_like_import_info();
    let all_import = AllImport {
        dependencies: js_like_import.dependencies,
        imports: js_like_import
            .imports
            .into_iter()
            .map(|i| try_to_find_files_without_a_suffix(&i, &js_like_import.all_files))
            .chain(style_like_import.imports)
            .collect::<HashSet<_>>(),
    };
    let all_import_str = serde_json::to_string_pretty(&all_import)?;
    let mut file = File::create("minipp.report.json")?;
    file.write_all(all_import_str.as_bytes())?;
    println!("成功生成minipp.report.json文件！");
    let duration = start.elapsed(); // 计算耗时
    println!("Time elapsed: {:?}", duration);
    Ok(())
}
