/// Generates a standardized module tag from a Rust module path.
fn make_tag_from_module_path(module_path: &str) -> String {
    let cleaned = module_path.replace("::", ".");
    let cleaned = cleaned
        .trim_start_matches("crate.")
        .trim_start_matches("::");
    format!("collapse.module.{cleaned}")
}

/// Public wrapper for generating a module tag from a module path.
pub fn collapse_module_tag_from_module_path(module_path: &str) -> String {
    make_tag_from_module_path(module_path)
}

/// Generates a standardized module tag from a source file path.
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

/// Macro to generate a module tag for the current context.
///
/// By default, it uses the module path. If `file` is passed, it uses the file path.
#[macro_export]
macro_rules! collapse_tag {
    () => {
        $crate::core::utils::tags::collapse_module_tag_from_module_path(module_path!())
    };
    (file) => {
        $crate::core::utils::tags::collapse_module_tag_from_file(file!())
    };
}
