use minipp_rs::common::{MinippConfig, SUPPORT_FILE_TYPES_WITH_DOT};

fn main() {
    println!("b = {:?}", MinippConfig::default());
    println!("{:?}", SUPPORT_FILE_TYPES_WITH_DOT);
}
