use crate::util::{parse_size_rule, SizeAction, SizeOperator};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filter<T> {
    pub denies: Option<Vec<T>>,
    pub allows: Option<Vec<T>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectFilter {
    pub name: Option<Filter<String>>,
    pub id: Option<Filter<i64>>,
    pub default: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileFilter {
    pub type_filter: Option<Filter<String>>,
    pub extension_filter: Option<Filter<String>>,
    pub mime_type_filter: Option<Filter<String>>,
    pub size_rules: Option<Vec<String>>,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeniedFileInfo {
    pub name: String,
    pub size: u64,
    pub reason: String,
    pub file_type: String,
    pub extension: String,
    pub mime_type: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CourseFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermFilter {
    pub object_filter: Option<ObjectFilter>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filters {
    pub course_filter: Option<CourseFilter>,
    pub term_filter: Option<TermFilter>,
    pub file_filter: Option<FileFilter>,
}

pub fn get_file_type(extension: &str) -> String {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "image".to_string(),
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "wmv" => "video".to_string(),
        "mp3" | "wav" | "ogg" | "flac" | "aac" => "audio".to_string(),
        "pdf" | "doc" | "docx" | "ppt" | "pptx" | "xls" | "xlsx" | "txt" | "md" => {
            "document".to_string()
        }
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "archive".to_string(),
        _ => "other".to_string(),
    }
}

pub fn file_filter_check(
    f: &FileFilter,
    file_size: u64,
    file_extension: &str,
    file_mime_type: &str,
    file_name: &str,
    denied_files: &mut Vec<DeniedFileInfo>,
) -> bool {
    let mut default_setting = "allow";
    if let Some(ref x) = f.default {
        default_setting = x
    }

    let file_type = get_file_type(file_extension);

    if let Some(size_rules) = &f.size_rules {
        for rule_str in size_rules {
            if let Some(parsed_rule) = parse_size_rule(rule_str) {
                let matches = match parsed_rule.operator {
                    SizeOperator::GreaterThanOrEqual => file_size >= parsed_rule.size_bytes,
                    SizeOperator::LessThanOrEqual => file_size <= parsed_rule.size_bytes,
                    SizeOperator::GreaterThan => file_size > parsed_rule.size_bytes,
                    SizeOperator::LessThan => file_size < parsed_rule.size_bytes,
                };

                if matches {
                    match parsed_rule.action {
                        SizeAction::Deny => {
                            denied_files.push(DeniedFileInfo {
                                name: file_name.to_string(),
                                size: file_size,
                                reason: format!("Size rule: {} ", rule_str),
                                file_type: file_type.to_string(),
                                extension: file_extension.to_string(),
                                mime_type: file_mime_type.to_string(),
                            });
                            return false;
                        }
                        SizeAction::Allow => {}
                    }
                }
            }
        }
    }

    if let Some(ref type_filter) = f.type_filter {
        if let Some(denies) = &type_filter.denies {
            if denies.contains(&file_type) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!("Type filter (denies): Type {} is denied", file_type),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
        if let Some(allows) = &type_filter.allows {
            if !allows.contains(&file_type) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!("Type filter (allows): Type {} is not allowed", file_type),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
    }

    if let Some(ref extension_filter) = f.extension_filter {
        if let Some(denies) = &extension_filter.denies {
            if denies.contains(&file_extension.to_string()) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!(
                        "Extension filter (denies): Extension {} is denied",
                        file_extension
                    ),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
        if let Some(allows) = &extension_filter.allows {
            if !allows.contains(&file_extension.to_string()) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!(
                        "Extension filter (allows): Extension {} is not allowed",
                        file_extension
                    ),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
    }

    if let Some(ref mime_type_filter) = f.mime_type_filter {
        if let Some(denies) = &mime_type_filter.denies {
            if denies.contains(&file_mime_type.to_string()) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!(
                        "MIME type filter (denies): MIME type {} is denied",
                        file_mime_type
                    ),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
        if let Some(allows) = &mime_type_filter.allows {
            if !allows.contains(&file_mime_type.to_string()) {
                denied_files.push(DeniedFileInfo {
                    name: file_name.to_string(),
                    size: file_size,
                    reason: format!(
                        "MIME type filter (allows): MIME type {} is not allowed",
                        file_mime_type
                    ),
                    file_type: file_type.to_string(),
                    extension: file_extension.to_string(),
                    mime_type: file_mime_type.to_string(),
                });
                return false;
            }
        }
    }

    if default_setting == "deny" {
        let mut allowed_by_type = false;
        if let Some(ref type_filter) = f.type_filter {
            if let Some(allows) = &type_filter.allows {
                if allows.contains(&file_type) {
                    allowed_by_type = true;
                }
            }
        }

        let mut allowed_by_extension = false;
        if let Some(ref extension_filter) = f.extension_filter {
            if let Some(allows) = &extension_filter.allows {
                if allows.contains(&file_extension.to_string()) {
                    allowed_by_extension = true;
                }
            }
        }

        let mut allowed_by_mime_type = false;
        if let Some(ref mime_type_filter) = f.mime_type_filter {
            if let Some(allows) = &mime_type_filter.allows {
                if allows.contains(&file_mime_type.to_string()) {
                    allowed_by_mime_type = true;
                }
            }
        }

        let result = allowed_by_type || allowed_by_extension || allowed_by_mime_type;
        if !result {
            denied_files.push(DeniedFileInfo {
                name: file_name.to_string(),
                size: file_size,
                reason: "Default setting (deny): Did not match any allow rules".to_string(),
                file_type: file_type.to_string(),
                extension: file_extension.to_string(),
                mime_type: file_mime_type.to_string(),
            });
        }
        return result;
    } else {
        let mut type_filter_has_allows = false;
        if let Some(ref type_filter) = f.type_filter {
            if type_filter.allows.is_some() {
                type_filter_has_allows = true;
            }
        }
        if type_filter_has_allows
            && !f
                .type_filter
                .as_ref()
                .unwrap()
                .allows
                .as_ref()
                .unwrap()
                .contains(&file_type)
        {
            return false;
        }

        let mut extension_filter_has_allows = false;
        if let Some(ref extension_filter) = f.extension_filter {
            if extension_filter.allows.is_some() {
                extension_filter_has_allows = true;
            }
        }
        if extension_filter_has_allows
            && !f
                .extension_filter
                .as_ref()
                .unwrap()
                .allows
                .as_ref()
                .unwrap()
                .contains(&file_extension.to_string())
        {
            return false;
        }

        let mut mime_type_filter_has_allows = false;
        if let Some(ref mime_type_filter) = f.mime_type_filter {
            if mime_type_filter.allows.is_some() {
                mime_type_filter_has_allows = true;
            }
        }
        let result = !(mime_type_filter_has_allows
            && !f
                .mime_type_filter
                .as_ref()
                .unwrap()
                .allows
                .as_ref()
                .unwrap()
                .contains(&file_mime_type.to_string()));
        if !result {
            denied_files.push(DeniedFileInfo {
                name: file_name.to_string(),
                size: file_size,
                reason: format!(
                    "Default setting (allow): MIME type {} did not match allowed MIME types.",
                    file_mime_type
                ),
                file_type: file_type.to_string(),
                extension: file_extension.to_string(),
                mime_type: file_mime_type.to_string(),
            });
        }
        return result;
    }
}

pub fn object_filter_check(f: &ObjectFilter, id: i64, name: &String) -> bool {
    let mut default_setting = "allow";
    if let Some(ref x) = f.default {
        default_setting = x
    }
    if default_setting == "deny" {
        if let Some(ref x) = f.id {
            if x.allows.is_some() && x.allows.as_ref().unwrap().contains(&id) {
                return true;
            }
        };
        if let Some(ref x) = f.name {
            if x.allows.is_some() && x.allows.as_ref().unwrap().contains(name) {
                return true;
            }
        };
        return false;
    }

    if default_setting == "allow" {
        if let Some(ref x) = f.id {
            if x.denies.is_some() && x.denies.as_ref().unwrap().contains(&id) {
                return false;
            }
        };
        if let Some(ref x) = f.name {
            if x.denies.is_some() && x.denies.as_ref().unwrap().contains(name) {
                return false;
            }
        };
        return true;
    }
    true
}
