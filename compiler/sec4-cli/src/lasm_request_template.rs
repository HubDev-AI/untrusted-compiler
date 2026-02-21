use crate::{find_lasm_cookie_value, find_lasm_header_value, LasmRunRequest};
use std::collections::BTreeMap;

pub(crate) fn contains_lasm_request_placeholder_tokens(value: &str) -> bool {
    value.contains("{{req.pathParam:")
        || value.contains("{{req.header:")
        || value.contains("{{req.query:")
        || value.contains("{{req.cookie:")
        || value.contains("{{req.method}}")
        || value.contains("{{req.path}}")
        || value.contains("{{req.httpVersion}}")
        || value.contains("{{req.body}}")
        || value.contains("{{sanitizeHtml:req.pathParam:")
        || value.contains("{{sanitizeHtml:req.header:")
        || value.contains("{{sanitizeHtml:req.query:")
        || value.contains("{{sanitizeHtml:req.cookie:")
        || value.contains("{{sanitizeHtml:req.method}}")
        || value.contains("{{sanitizeHtml:req.path}}")
        || value.contains("{{sanitizeHtml:req.httpVersion}}")
        || value.contains("{{sanitizeHtml:req.body}}")
}

pub(crate) fn escape_lasm_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn materialize_lasm_request_placeholders(
    value: &str,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) -> String {
    let with_method = value.replace("{{req.method}}", request.method.as_str());
    let with_path = with_method.replace("{{req.path}}", request.path.as_str());
    let with_http_version = with_path.replace("{{req.httpVersion}}", request.http_version.as_str());
    let request_body = String::from_utf8_lossy(&request.body);
    let with_body = with_http_version.replace("{{req.body}}", request_body.as_ref());
    let with_path_params =
        replace_lasm_response_placeholder_tokens(&with_body, "{{req.pathParam:", |key| {
            path_params.get(key.trim()).cloned()
        });
    let with_headers =
        replace_lasm_response_placeholder_tokens(&with_path_params, "{{req.header:", |key| {
            find_lasm_header_value(&request.headers, key.trim()).map(ToOwned::to_owned)
        });
    let with_cookies =
        replace_lasm_response_placeholder_tokens(&with_headers, "{{req.cookie:", |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
        });
    let with_queries =
        replace_lasm_response_placeholder_tokens(&with_cookies, "{{req.query:", |key| {
            request.query_params.get(key.trim()).cloned()
        });

    let escaped_method = escape_lasm_html(request.method.as_str());
    let escaped_path = escape_lasm_html(request.path.as_str());
    let escaped_http_version = escape_lasm_html(request.http_version.as_str());
    let escaped_body = escape_lasm_html(request_body.as_ref());

    let with_sanitized_method =
        with_queries.replace("{{sanitizeHtml:req.method}}", escaped_method.as_str());
    let with_sanitized_path =
        with_sanitized_method.replace("{{sanitizeHtml:req.path}}", escaped_path.as_str());
    let with_sanitized_http_version = with_sanitized_path.replace(
        "{{sanitizeHtml:req.httpVersion}}",
        escaped_http_version.as_str(),
    );
    let with_sanitized_body =
        with_sanitized_http_version.replace("{{sanitizeHtml:req.body}}", escaped_body.as_str());
    let with_sanitized_path_params = replace_lasm_response_placeholder_tokens(
        &with_sanitized_body,
        "{{sanitizeHtml:req.pathParam:",
        |key| {
            path_params
                .get(key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    let with_sanitized_headers = replace_lasm_response_placeholder_tokens(
        &with_sanitized_path_params,
        "{{sanitizeHtml:req.header:",
        |key| find_lasm_header_value(&request.headers, key.trim()).map(escape_lasm_html),
    );
    let with_sanitized_cookies = replace_lasm_response_placeholder_tokens(
        &with_sanitized_headers,
        "{{sanitizeHtml:req.cookie:",
        |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    replace_lasm_response_placeholder_tokens(
        &with_sanitized_cookies,
        "{{sanitizeHtml:req.query:",
        |key| {
            request
                .query_params
                .get(key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    )
}

fn replace_lasm_response_placeholder_tokens(
    body: &str,
    prefix: &str,
    resolve: impl Fn(&str) -> Option<String>,
) -> String {
    let mut output = String::with_capacity(body.len());
    let mut rest = body;
    loop {
        let Some(start) = rest.find(prefix) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let value_start = start + prefix.len();
        let after_value_start = &rest[value_start..];
        let Some(value_end) = after_value_start.find("}}") else {
            output.push_str(&rest[start..]);
            break;
        };
        let token_value = &after_value_start[..value_end];
        if let Some(value) = resolve(token_value) {
            output.push_str(value.as_str());
        }
        rest = &after_value_start[value_end + 2..];
    }
    output
}
