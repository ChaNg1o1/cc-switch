use crate::provider::Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalEndpoint {
    ResponsesCreate,
    ResponsesCompact,
    ChatCompletions,
    ClaudeMessages,
}

impl LogicalEndpoint {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ResponsesCreate => "ResponsesCreate",
            Self::ResponsesCompact => "ResponsesCompact",
            Self::ChatCompletions => "ChatCompletions",
            Self::ClaudeMessages => "ClaudeMessages",
        }
    }

    pub fn default_path(self) -> &'static str {
        match self {
            Self::ResponsesCreate => "/v1/responses",
            Self::ResponsesCompact => "/v1/responses/compact",
            Self::ChatCompletions => "/v1/chat/completions",
            Self::ClaudeMessages => "/v1/messages",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamApiStyle {
    StandardOpenAi,
    PrefixedOpenAi,
    Custom,
}

impl UpstreamApiStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StandardOpenAi => "standard_openai",
            Self::PrefixedOpenAi => "prefixed_openai",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamResolution {
    pub url: String,
    pub logical_endpoint: LogicalEndpoint,
    pub applied_style: UpstreamApiStyle,
    pub path_rewritten: bool,
    pub endpoint_path: String,
}

pub fn resolve_upstream_url(
    provider: &Provider,
    base_url: &str,
    logical_endpoint: LogicalEndpoint,
) -> UpstreamResolution {
    let meta = provider.meta.as_ref();
    let applied_style = match meta.and_then(|value| value.upstream_api_style.as_deref()) {
        Some("prefixed_openai") => UpstreamApiStyle::PrefixedOpenAi,
        Some("custom") => UpstreamApiStyle::Custom,
        _ => UpstreamApiStyle::StandardOpenAi,
    };

    let openai_base_path = meta
        .and_then(|value| value.openai_base_path.as_deref())
        .map(normalize_path_segment)
        .unwrap_or_else(|| match applied_style {
            UpstreamApiStyle::PrefixedOpenAi => meta
                .and_then(|value| value.upstream_prefix.as_deref())
                .map(normalize_path_segment)
                .unwrap_or_else(|| "/openai".to_string()),
            _ => String::new(),
        });

    let configured_path = match logical_endpoint {
        LogicalEndpoint::ResponsesCreate => meta.and_then(|value| value.responses_path.as_deref()),
        LogicalEndpoint::ResponsesCompact => meta
            .and_then(|value| value.responses_compact_path.as_deref()),
        LogicalEndpoint::ChatCompletions => meta
            .and_then(|value| value.chat_completions_path.as_deref()),
        LogicalEndpoint::ClaudeMessages => None,
    };

    let endpoint_path = if logical_endpoint == LogicalEndpoint::ClaudeMessages {
        logical_endpoint.default_path().to_string()
    } else {
        join_paths(&openai_base_path, configured_path.unwrap_or(logical_endpoint.default_path()))
    };

    let url = build_url(base_url, &endpoint_path);
    let path_rewritten = endpoint_path != logical_endpoint.default_path();

    UpstreamResolution {
        url,
        logical_endpoint,
        applied_style,
        path_rewritten,
        endpoint_path,
    }
}

fn normalize_path_segment(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return String::new();
    }

    let mut normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    while normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

fn join_paths(prefix: &str, path: &str) -> String {
    let prefix = normalize_path_segment(prefix);
    let path = normalize_path_segment(path);

    let joined = if prefix.is_empty() {
        if path.is_empty() {
            "/".to_string()
        } else {
            path
        }
    } else if path.is_empty() {
        prefix
    } else {
        format!("{prefix}{path}")
    };

    collapse_duplicate_versions(&joined)
}

fn build_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    collapse_duplicate_versions(&format!("{base}/{path}"))
}

fn collapse_duplicate_versions(value: &str) -> String {
    let mut normalized = value.to_string();
    while normalized.contains("/v1/v1") {
        normalized = normalized.replace("/v1/v1", "/v1");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Provider, ProviderMeta};
    use serde_json::json;

    fn provider_with_meta(meta: ProviderMeta) -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test".to_string(),
            settings_config: json!({}),
            website_url: None,
            category: Some("codex".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: Some(meta),
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn standard_openai_origin_builds_v1_responses() {
        let provider = provider_with_meta(ProviderMeta::default());
        let resolved = resolve_upstream_url(
            &provider,
            "https://api.openai.com",
            LogicalEndpoint::ResponsesCreate,
        );

        assert_eq!(resolved.url, "https://api.openai.com/v1/responses");
        assert_eq!(resolved.applied_style, UpstreamApiStyle::StandardOpenAi);
        assert!(!resolved.path_rewritten);
    }

    #[test]
    fn prefixed_openai_uses_upstream_prefix() {
        let provider = provider_with_meta(ProviderMeta {
            upstream_api_style: Some("prefixed_openai".to_string()),
            upstream_prefix: Some("/openai".to_string()),
            ..ProviderMeta::default()
        });

        let resolved = resolve_upstream_url(
            &provider,
            "https://gateway.example.com",
            LogicalEndpoint::ResponsesCreate,
        );

        assert_eq!(resolved.url, "https://gateway.example.com/openai/v1/responses");
        assert!(resolved.path_rewritten);
    }

    #[test]
    fn custom_paths_override_defaults() {
        let provider = provider_with_meta(ProviderMeta {
            upstream_api_style: Some("custom".to_string()),
            openai_base_path: Some("/openai".to_string()),
            responses_compact_path: Some("/v2/responses/compact".to_string()),
            ..ProviderMeta::default()
        });

        let resolved = resolve_upstream_url(
            &provider,
            "https://gateway.example.com",
            LogicalEndpoint::ResponsesCompact,
        );

        assert_eq!(
            resolved.url,
            "https://gateway.example.com/openai/v2/responses/compact"
        );
        assert_eq!(resolved.endpoint_path, "/openai/v2/responses/compact");
        assert!(resolved.path_rewritten);
    }

    #[test]
    fn existing_v1_base_deduplicates_version() {
        let provider = provider_with_meta(ProviderMeta::default());
        let resolved = resolve_upstream_url(
            &provider,
            "https://api.openai.com/v1",
            LogicalEndpoint::ChatCompletions,
        );

        assert_eq!(resolved.url, "https://api.openai.com/v1/chat/completions");
    }
}