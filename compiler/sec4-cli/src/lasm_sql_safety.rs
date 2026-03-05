pub(crate) fn has_lasm_sql_non_trailing_statement_separator(query_template: &str) -> bool {
    let bytes = query_template.as_bytes();
    let mut index = 0usize;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while index < bytes.len() {
        if in_line_comment {
            if bytes[index] == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                block_comment_depth += 1;
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if in_single_quote {
            if bytes[index] == b'\'' {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }
        if in_double_quote {
            if bytes[index] == b'"' {
                if index + 1 < bytes.len() && bytes[index + 1] == b'"' {
                    index += 2;
                    continue;
                }
                in_double_quote = false;
            }
            index += 1;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-' {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'\'' {
            in_single_quote = true;
            index += 1;
            continue;
        }
        if bytes[index] == b'"' {
            in_double_quote = true;
            index += 1;
            continue;
        }
        if bytes[index] == b';' && !query_template[index + 1..].trim().is_empty() {
            return true;
        }
        index += 1;
    }
    false
}
