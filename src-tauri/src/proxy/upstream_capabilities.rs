use super::providers::LogicalEndpoint;
use crate::{app_config::AppType, provider::Provider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    #[default]
    Unknown,
}

impl CapabilitySupport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponsesCompactMode {
    Native,
    #[default]
    Synthetic,
    Disabled,
}

impl ResponsesCompactMode {
    pub fn from_provider(provider: &Provider) -> Self {
        match provider
            .meta
            .as_ref()
            .and_then(|meta| meta.responses_compact_mode.as_deref())
        {
            Some("native") => Self::Native,
            Some("disabled") => Self::Disabled,
            Some("synthetic") => Self::Synthetic,
            _ => Self::Synthetic,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Synthetic => "synthetic",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamCompatibility {
    pub responses: CapabilitySupport,
    pub responses_compact: CapabilitySupport,
    pub streaming_sse: CapabilitySupport,
    pub responses_schema: CapabilitySupport,
    pub compact_mode: ResponsesCompactMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl UpstreamCompatibility {
    pub fn for_provider(app_type: &AppType, provider: &Provider) -> Self {
        let compact_mode = ResponsesCompactMode::from_provider(provider);
        let mut snapshot = Self {
            responses: CapabilitySupport::Unknown,
            responses_compact: CapabilitySupport::Unknown,
            streaming_sse: CapabilitySupport::Unknown,
            responses_schema: CapabilitySupport::Unknown,
            compact_mode,
            last_checked_at: None,
            source: None,
            note: None,
        };

        if matches!(app_type, AppType::Claude) {
            let api_format = provider
                .meta
                .as_ref()
                .and_then(|meta| meta.api_format.as_deref())
                .or_else(|| {
                    provider
                        .settings_config
                        .get("api_format")
                        .and_then(|value| value.as_str())
                })
                .unwrap_or("anthropic");

            if api_format == "anthropic" {
                snapshot.responses = CapabilitySupport::Unsupported;
                snapshot.responses_compact = CapabilitySupport::Unsupported;
                snapshot.responses_schema = CapabilitySupport::Unsupported;
                snapshot.note = Some("anthropic_native_messages".to_string());
            }
        }

        snapshot
    }

    pub fn with_success(
        mut self,
        logical_endpoint: LogicalEndpoint,
        streaming_sse: Option<bool>,
        source: &str,
    ) -> Self {
        match logical_endpoint {
            LogicalEndpoint::ResponsesCreate => {
                self.responses = CapabilitySupport::Supported;
                self.responses_schema = CapabilitySupport::Supported;
            }
            LogicalEndpoint::ResponsesCompact => {
                self.responses_compact = CapabilitySupport::Supported;
            }
            LogicalEndpoint::ChatCompletions | LogicalEndpoint::ClaudeMessages => {}
        }

        if let Some(is_supported) = streaming_sse {
            self.streaming_sse = if is_supported {
                CapabilitySupport::Supported
            } else {
                CapabilitySupport::Unsupported
            };
        }

        self.last_checked_at = Some(chrono::Utc::now().timestamp());
        self.source = Some(source.to_string());
        self.note = None;
        self
    }

    pub fn with_http_failure(
        mut self,
        logical_endpoint: LogicalEndpoint,
        status: u16,
        source: &str,
        note: Option<String>,
    ) -> Self {
        match logical_endpoint {
            LogicalEndpoint::ResponsesCreate => {
                if matches!(status, 404 | 405 | 501) {
                    self.responses = CapabilitySupport::Unsupported;
                } else if matches!(status, 400 | 422) {
                    self.responses = CapabilitySupport::Supported;
                    self.responses_schema = CapabilitySupport::Unsupported;
                }
            }
            LogicalEndpoint::ResponsesCompact => {
                if matches!(status, 404 | 405 | 501) {
                    self.responses_compact = CapabilitySupport::Unsupported;
                } else if matches!(status, 400 | 422) {
                    self.responses_compact = CapabilitySupport::Supported;
                }
            }
            LogicalEndpoint::ChatCompletions | LogicalEndpoint::ClaudeMessages => {}
        }

        self.last_checked_at = Some(chrono::Utc::now().timestamp());
        self.source = Some(source.to_string());
        self.note = note;
        self
    }

    pub fn summary(&self) -> String {
        format!(
            "responses={}, compact={}, sse={}, schema={}, compact_mode={}",
            self.responses.as_str(),
            self.responses_compact.as_str(),
            self.streaming_sse.as_str(),
            self.responses_schema.as_str(),
            self.compact_mode.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Provider, ProviderMeta};
    use serde_json::json;

    fn build_provider(meta: Option<ProviderMeta>) -> Provider {
        Provider {
            id: "provider-1".to_string(),
            name: "Provider 1".to_string(),
            settings_config: json!({}),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn compact_mode_defaults_to_synthetic() {
        let provider = build_provider(None);
        assert_eq!(ResponsesCompactMode::from_provider(&provider), ResponsesCompactMode::Synthetic);
    }

    #[test]
    fn compact_mode_respects_provider_meta() {
        let provider = build_provider(Some(ProviderMeta {
            responses_compact_mode: Some("native".to_string()),
            ..Default::default()
        }));
        assert_eq!(ResponsesCompactMode::from_provider(&provider), ResponsesCompactMode::Native);
    }

    #[test]
    fn anthropic_native_starts_with_responses_unsupported() {
        let provider = build_provider(Some(ProviderMeta {
            api_format: Some("anthropic".to_string()),
            ..Default::default()
        }));

        let snapshot = UpstreamCompatibility::for_provider(&AppType::Claude, &provider);

        assert_eq!(snapshot.responses, CapabilitySupport::Unsupported);
        assert_eq!(snapshot.responses_compact, CapabilitySupport::Unsupported);
        assert_eq!(snapshot.responses_schema, CapabilitySupport::Unsupported);
    }

    #[test]
    fn responses_404_marks_endpoint_unsupported() {
        let provider = build_provider(None);
        let snapshot = UpstreamCompatibility::for_provider(&AppType::Codex, &provider)
            .with_http_failure(
                LogicalEndpoint::ResponsesCompact,
                404,
                "test",
                Some("not found".to_string()),
            );

        assert_eq!(snapshot.responses_compact, CapabilitySupport::Unsupported);
    }

    #[test]
    fn responses_422_marks_schema_not_supported_but_endpoint_supported() {
        let provider = build_provider(None);
        let snapshot = UpstreamCompatibility::for_provider(&AppType::Codex, &provider)
            .with_http_failure(
                LogicalEndpoint::ResponsesCreate,
                422,
                "test",
                Some("schema mismatch".to_string()),
            );

        assert_eq!(snapshot.responses, CapabilitySupport::Supported);
        assert_eq!(snapshot.responses_schema, CapabilitySupport::Unsupported);
    }
}
