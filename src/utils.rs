use std::collections::HashMap;
use itertools::Itertools;

pub fn get_path(path_parts: Vec<&str>, args: Vec<String>) -> Option<String> {
    let file_name = if path_parts.len() > 2 {
        path_parts[2]
    } else {
        ""
    };

    let file_path =
        if let Some((index, _)) = &args.iter().find_position(|arg| arg.contains("--directory")) {
            &args[index + 1]
        } else {
            ""
        };

    if file_name.is_empty() || file_path.is_empty() {
        return None;
    }

    Some(format!("{}/{}", file_path, file_name))
}

pub fn extract_headers(headers: &[String]) -> HashMap<String, String> {
    let headers = headers
        .iter()
        .filter(|s| s.contains(": "))
        .map(|s| {
            let parts = s.split(": ").collect::<Vec<&str>>();
            (parts[0].to_lowercase(), parts[1].to_lowercase())
        }).collect::<Vec<(String, String)>>();

    HashMap::from_iter(headers.iter().cloned())
}