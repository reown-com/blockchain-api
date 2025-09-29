use {
    crate::state::AppState, axum::response::Response, hyper::header, std::sync::Arc, tracing::error,
};

/// CORS default allowed origins
pub const CORS_ALLOWED_ORIGINS: [&str; 1] = ["http://localhost:3000"];

pub fn insert_cors_headers(response: &mut Response, origin: &str) {
    let headers = response.headers_mut();
    // Strip CR/LF to avoid header injection
    let cleaned_origin = origin.replace(['\r', '\n'], "");
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        match header::HeaderValue::from_str(&cleaned_origin) {
            Ok(value) => value,
            Err(e) => {
                // Don't set CORS headers for invalid origins
                error!("Invalid origin header value: {origin}, {e}");
                return;
            }
        },
    );
    headers.insert(header::VARY, header::HeaderValue::from_static("Origin"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static(
            "content-type, user-agent, referer, origin, access-control-request-method, access-control-request-headers, solana-client, sec-fetch-mode, x-sdk-type, x-sdk-version",
        ),
    );
}

pub fn insert_cors_allow_all_headers(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static(
            "content-type, user-agent, referer, origin, access-control-request-method, access-control-request-headers, solana-client, sec-fetch-mode, x-sdk-type, x-sdk-version",
        ),
    );
}

/// Match a hostname against a host pattern which may start with `*.` for subdomains.
/// Examples:
/// - pattern `*.` + suffix `test.app` matches `a.test.app`, `b.c.test.app`, but not `test.app`
/// - pattern `example.com` matches only `example.com`
/// - pattern `*` matches any host
pub fn host_matches_pattern(pattern_lc: &str, host_lc: &str) -> bool {
    if pattern_lc == "*" {
        return true;
    }
    if let Some(suffix) = pattern_lc.strip_prefix("*.") {
        if !host_lc.ends_with(suffix) {
            return false;
        }
        // Ensure there is a subdomain component (i.e., not an exact match)
        let prefix_len = host_lc.len().saturating_sub(suffix.len());
        if prefix_len == 0 {
            return false;
        }
        // The character just before suffix must be a dot
        let dot_index = prefix_len.saturating_sub(1);
        return host_lc.as_bytes().get(dot_index) == Some(&b'.');
    }
    pattern_lc == host_lc
}

pub async fn get_project_allowed_origins(
    state: Arc<AppState>,
    project_id: &str,
) -> Option<Vec<String>> {
    let project = state.registry.project_data(project_id).await.ok()?;
    let mut allowed_origins: Vec<String> = Vec::new();
    allowed_origins.extend(project.data.allowed_origins.into_iter());
    // Deduplicate, case-insensitive
    allowed_origins.sort_by_key(|s| s.to_ascii_lowercase());
    allowed_origins.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    // Append default allowed origins
    allowed_origins.extend(CORS_ALLOWED_ORIGINS.iter().map(|s| s.to_string()));
    Some(allowed_origins)
}

pub fn insert_allowed_origins_debug_header(response: &mut Response, list: &[String]) {
    // Sanitize each origin by stripping CR/LF and keeping only valid header values
    let sanitized: Vec<String> = list
        .iter()
        .map(|s| s.replace(['\r', '\n'], ""))
        .filter(|s| header::HeaderValue::from_str(s).is_ok())
        .collect();

    if sanitized.is_empty() {
        return;
    }
    if let Ok(value) = header::HeaderValue::from_str(&sanitized.join(",")) {
        response
            .headers_mut()
            .insert(header::HeaderName::from_static("x-allowed-origins"), value);
    }
}
