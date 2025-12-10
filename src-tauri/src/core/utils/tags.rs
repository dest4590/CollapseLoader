fn make_tag_from_module_path(module_path: &str) -> String {
    let cleaned = module_path.replace("::", ".");
    let cleaned = cleaned
        .trim_start_matches("crate.")
        .trim_start_matches("::");
    format!("collapse.module.{cleaned}")
}

pub fn collapse_module_tag_from_module_path(module_path: &str) -> String {
    make_tag_from_module_path(module_path)
}

pub fn collapse_module_tag_from_file(file_path: &str) -> String {
    let mut cleaned = file_path.replace("\\", "/");
    cleaned = cleaned.trim_start_matches("./").to_string();
    cleaned = cleaned
        .trim_start_matches("crate/")
        .trim_start_matches("/")
        .to_string();
    let cleaned = cleaned.replace('/', ".").replace('.', "_");
    format!("collapse.module.{cleaned}")
}

#[macro_export]
macro_rules! collapse_tag {
    () => {
        $crate::core::utils::tags::collapse_module_tag_from_module_path(module_path!())
    };
    (file) => {
        $crate::core::utils::tags::collapse_module_tag_from_file(file!())
    };
}
