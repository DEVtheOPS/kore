use std::collections::HashSet;

const MAX_CLUSTER_NAME_LEN: usize = 100;
const MAX_CONTEXT_NAME_LEN: usize = 253;
const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_TAGS_COUNT: usize = 20;
const MAX_TAG_LEN: usize = 32;

fn is_allowed_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            ' ' | '-' | '_' | '.' | ':' | '/' | '@' | '+' | '(' | ')' | '[' | ']'
        )
}

fn is_allowed_tag_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':' | '/')
}

pub fn validate_cluster_name(name: String) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Cluster name cannot be empty".to_string());
    }
    if trimmed.len() > MAX_CLUSTER_NAME_LEN {
        return Err(format!(
            "Cluster name must be {} characters or fewer",
            MAX_CLUSTER_NAME_LEN
        ));
    }
    if !trimmed.chars().all(is_allowed_name_char) {
        return Err(
            "Cluster name contains invalid characters. Allowed: letters, numbers, space, - _ . : / @ + ( ) [ ]"
                .to_string(),
        );
    }

    Ok(trimmed.to_string())
}

pub fn validate_context_name(context_name: String) -> Result<String, String> {
    let trimmed = context_name.trim();
    if trimmed.is_empty() {
        return Err("Context name cannot be empty".to_string());
    }
    if trimmed.len() > MAX_CONTEXT_NAME_LEN {
        return Err(format!(
            "Context name must be {} characters or fewer",
            MAX_CONTEXT_NAME_LEN
        ));
    }
    if !trimmed.chars().all(is_allowed_name_char) {
        return Err(
            "Context name contains invalid characters. Allowed: letters, numbers, space, - _ . : / @ + ( ) [ ]"
                .to_string(),
        );
    }

    Ok(trimmed.to_string())
}

pub fn validate_description(description: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = description else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > MAX_DESCRIPTION_LEN {
        return Err(format!(
            "Description must be {} characters or fewer",
            MAX_DESCRIPTION_LEN
        ));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err("Description contains invalid control characters".to_string());
    }

    Ok(Some(trimmed.to_string()))
}

pub fn validate_tags(tags: Vec<String>) -> Result<Vec<String>, String> {
    if tags.len() > MAX_TAGS_COUNT {
        return Err(format!("At most {} tags are allowed", MAX_TAGS_COUNT));
    }

    let mut seen = HashSet::new();
    let mut validated = Vec::with_capacity(tags.len());

    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Err("Tags cannot be empty".to_string());
        }
        if trimmed.len() > MAX_TAG_LEN {
            return Err(format!(
                "Tag '{}' exceeds {} characters",
                trimmed, MAX_TAG_LEN
            ));
        }
        if !trimmed.chars().all(is_allowed_tag_char) {
            return Err(format!(
                "Tag '{}' contains invalid characters. Allowed: letters, numbers, - _ . : /",
                trimmed
            ));
        }
        if !seen.insert(trimmed.to_string()) {
            return Err(format!("Duplicate tag '{}'", trimmed));
        }
        validated.push(trimmed.to_string());
    }

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_name_is_trimmed_and_validated() {
        let validated = validate_cluster_name("  prod-cluster  ".to_string()).unwrap();
        assert_eq!(validated, "prod-cluster");
    }

    #[test]
    fn cluster_name_rejects_invalid_characters() {
        let err = validate_cluster_name("prod\ncluster".to_string()).unwrap_err();
        assert!(err.contains("invalid characters"));
    }

    #[test]
    fn description_allows_none_or_blank() {
        assert_eq!(validate_description(None).unwrap(), None);
        assert_eq!(validate_description(Some("   ".to_string())).unwrap(), None);
    }

    #[test]
    fn description_rejects_control_chars() {
        let err = validate_description(Some("hello\u{0007}".to_string())).unwrap_err();
        assert!(err.contains("control"));
    }

    #[test]
    fn tags_are_trimmed_and_deduplicated() {
        let tags = validate_tags(vec!["  prod ".to_string(), "team-a".to_string()]).unwrap();
        assert_eq!(tags, vec!["prod".to_string(), "team-a".to_string()]);
    }

    #[test]
    fn tags_reject_duplicates() {
        let err = validate_tags(vec!["prod".to_string(), "prod".to_string()]).unwrap_err();
        assert!(err.contains("Duplicate"));
    }
}
