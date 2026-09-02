use super::call::parse_call;
use crate::literal::LiteralValue;
use crate::manifest::{DestinationMode, RenderPolicy};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(super) fn parse_render_policy(rest: &str) -> PrayResult<RenderPolicy> {
    let (_, keywords) = parse_call(rest)?;
    for key in keywords.keys() {
        if !matches!(key.as_str(), "mode" | "conflict" | "churn" | "header") {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!("render does not accept {key}"),
            });
        }
    }
    Ok(RenderPolicy {
        mode: keywords
            .get("mode")
            .and_then(|value| value.as_string())
            .unwrap_or("managed")
            .to_string(),
        conflict: keywords
            .get("conflict")
            .and_then(|value| value.as_string())
            .unwrap_or("fail")
            .to_string(),
        churn: keywords
            .get("churn")
            .and_then(|value| value.as_string())
            .unwrap_or("minimal")
            .to_string(),
        header: keywords
            .get("header")
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
    })
}

pub(super) fn destination_header_keyword(
    mode: DestinationMode,
    keywords: &BTreeMap<String, LiteralValue>,
) -> PrayResult<Option<bool>> {
    let label = match mode {
        DestinationMode::Compose => "compose",
        DestinationMode::Tree => "tree",
        DestinationMode::Legacy => "destination",
    };
    for key in keywords.keys() {
        if key != "header" {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!("{label} does not accept {key}"),
            });
        }
    }
    let Some(value) = keywords.get("header") else {
        return Ok(None);
    };
    if mode != DestinationMode::Compose {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: format!("{label} does not accept header"),
        });
    }
    value.as_bool().map(Some).ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "header must be true or false".to_string(),
    })
}
