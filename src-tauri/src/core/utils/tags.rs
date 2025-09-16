use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static TAG_CACHE: Lazy<Mutex<HashMap<String, &'static str>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn make_tag_from_module_path(module_path: &str) -> String {
    let cleaned = module_path.replace("::", ".");
    let cleaned = cleaned
        .trim_start_matches("crate.")
        .trim_start_matches("::");
    format!("collapse.module.{cleaned}")
}

pub fn collapse_module_tag_cached_from_module_path(module_path: &str) -> &'static str {
    let key = make_tag_from_module_path(module_path);
    let mut cache = TAG_CACHE.lock().unwrap();
    if let Some(&v) = cache.get(&key) {
        return v;
    }
    let leaked: &'static str = Box::leak(key.into_boxed_str());
    cache.insert(leaked.to_string(), leaked);
    leaked
}

#[macro_export]
macro_rules! collapse_tag {
    () => {
        $crate::core::utils::tags::collapse_module_tag_cached_from_module_path(module_path!())
    };
    (file) => {
        $crate::core::utils::tags::collapse_module_tag_cached_from_file(file!())
    };
}
