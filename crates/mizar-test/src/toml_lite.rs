use std::collections::BTreeMap;
use std::iter::Peekable;
use std::str::Lines;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<TomlValue>),
}

pub type TomlTable = BTreeMap<String, TomlValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    pub line: usize,
    pub message: String,
}

impl TomlError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

pub fn parse_table(input: &str) -> Result<TomlTable, TomlError> {
    let mut table = TomlTable::new();
    let mut lines = input.lines().enumerate().peekable();
    while let Some((line_idx, raw_line)) = lines.next() {
        let line_no = line_idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            return Err(TomlError::new(
                line_no,
                "tables are not allowed in expectation sidecars",
            ));
        }
        let (key, raw_value) = split_key_value(line, line_no)?;
        let value = parse_value_with_continuation(raw_value, line_no, &mut lines)?;
        if table.insert(key.to_owned(), value).is_some() {
            return Err(TomlError::new(line_no, format!("duplicate key `{key}`")));
        }
    }
    Ok(table)
}

pub fn parse_expectation_tables(
    input: &str,
) -> Result<(TomlTable, Option<TomlTable>, Vec<TomlTable>), TomlError> {
    let mut root = TomlTable::new();
    let mut origin = None;
    let mut tokens = Vec::new();
    let mut current_token: Option<TomlTable> = None;
    let mut current_table = ExpectationTable::Root;
    let mut lines = input.lines().enumerate().peekable();

    while let Some((line_idx, raw_line)) = lines.next() {
        let line_no = line_idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[tokens]]" {
            if let Some(token) = current_token.take() {
                tokens.push(token);
            }
            current_token = Some(TomlTable::new());
            current_table = ExpectationTable::Token;
            continue;
        }
        if line == "[origin]" {
            if origin.is_some() {
                return Err(TomlError::new(
                    line_no,
                    "duplicate expectation table `[origin]`",
                ));
            }
            if let Some(token) = current_token.take() {
                tokens.push(token);
            }
            origin = Some(TomlTable::new());
            current_table = ExpectationTable::Origin;
            continue;
        }
        if line.starts_with('[') {
            return Err(TomlError::new(
                line_no,
                format!("unsupported expectation table `{line}`"),
            ));
        }

        let (key, raw_value) = split_key_value(line, line_no)?;
        let value = parse_value_with_continuation(raw_value, line_no, &mut lines)?;
        let table = match current_table {
            ExpectationTable::Root => &mut root,
            ExpectationTable::Origin => origin.as_mut().expect("origin table exists when selected"),
            ExpectationTable::Token => current_token
                .as_mut()
                .expect("token table exists when selected"),
        };
        if table.insert(key.to_owned(), value).is_some() {
            return Err(TomlError::new(line_no, format!("duplicate key `{key}`")));
        }
    }

    if let Some(token) = current_token {
        tokens.push(token);
    }
    Ok((root, origin, tokens))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectationTable {
    Root,
    Origin,
    Token,
}

pub fn parse_requirement_tables(input: &str) -> Result<Vec<TomlTable>, TomlError> {
    let mut records = Vec::new();
    let mut current: Option<TomlTable> = None;
    let mut lines = input.lines().enumerate().peekable();

    while let Some((line_idx, raw_line)) = lines.next() {
        let line_no = line_idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[requirement]]" {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some(TomlTable::new());
            continue;
        }
        if line.starts_with('[') {
            return Err(TomlError::new(
                line_no,
                format!("unsupported table `{line}`"),
            ));
        }
        let Some(record) = current.as_mut() else {
            return Err(TomlError::new(
                line_no,
                "manifest keys must be inside [[requirement]]",
            ));
        };
        let (key, raw_value) = split_key_value(line, line_no)?;
        let value = parse_value_with_continuation(raw_value, line_no, &mut lines)?;
        if record.insert(key.to_owned(), value).is_some() {
            return Err(TomlError::new(line_no, format!("duplicate key `{key}`")));
        }
    }

    if let Some(record) = current {
        records.push(record);
    }
    Ok(records)
}

pub fn required_string(table: &TomlTable, key: &str) -> Result<String, String> {
    match table.get(key) {
        Some(TomlValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(format!("`{key}` must be a string")),
        None => Err(format!("missing `{key}`")),
    }
}

pub fn optional_string(table: &TomlTable, key: &str) -> Result<Option<String>, String> {
    match table.get(key) {
        Some(TomlValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("`{key}` must be a string")),
        None => Ok(None),
    }
}

pub fn required_u32(table: &TomlTable, key: &str) -> Result<u32, String> {
    match table.get(key) {
        Some(TomlValue::Integer(value)) if *value >= 0 && *value <= u32::MAX as i64 => {
            Ok(*value as u32)
        }
        Some(_) => Err(format!("`{key}` must be a non-negative integer")),
        None => Err(format!("missing `{key}`")),
    }
}

pub fn optional_u32(table: &TomlTable, key: &str) -> Result<Option<u32>, String> {
    match table.get(key) {
        Some(TomlValue::Integer(value)) if *value >= 0 && *value <= u32::MAX as i64 => {
            Ok(Some(*value as u32))
        }
        Some(_) => Err(format!("`{key}` must be a non-negative integer")),
        None => Ok(None),
    }
}

pub fn required_bool(table: &TomlTable, key: &str) -> Result<bool, String> {
    match table.get(key) {
        Some(TomlValue::Boolean(value)) => Ok(*value),
        Some(_) => Err(format!("`{key}` must be a boolean")),
        None => Err(format!("missing `{key}`")),
    }
}

pub fn optional_bool(table: &TomlTable, key: &str) -> Result<Option<bool>, String> {
    match table.get(key) {
        Some(TomlValue::Boolean(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("`{key}` must be a boolean")),
        None => Ok(None),
    }
}

pub fn string_array(table: &TomlTable, key: &str) -> Result<Vec<String>, String> {
    match table.get(key) {
        Some(TomlValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                TomlValue::String(item) => Ok(item.clone()),
                _ => Err(format!("`{key}` must contain only strings")),
            })
            .collect(),
        Some(_) => Err(format!("`{key}` must be an array of strings")),
        None => Err(format!("missing `{key}`")),
    }
}

fn split_key_value(line: &str, line_no: usize) -> Result<(&str, &str), TomlError> {
    let Some((key, value)) = line.split_once('=') else {
        return Err(TomlError::new(line_no, "expected `key = value`"));
    };
    let key = key.trim();
    if key.is_empty() {
        return Err(TomlError::new(line_no, "empty key"));
    }
    Ok((key, value.trim()))
}

fn parse_value_with_continuation<'a>(
    raw_value: &str,
    line_no: usize,
    lines: &mut Peekable<std::iter::Enumerate<Lines<'a>>>,
) -> Result<TomlValue, TomlError> {
    if raw_value.trim_start().starts_with('[') && !raw_value.trim_end().ends_with(']') {
        let mut merged = String::from(raw_value);
        for (_, continuation) in lines.by_ref() {
            let stripped = strip_comment(continuation);
            merged.push('\n');
            merged.push_str(stripped);
            if stripped.trim_end().ends_with(']') {
                break;
            }
        }
        parse_value(&merged, line_no)
    } else {
        parse_value(raw_value, line_no)
    }
}

fn parse_value(value: &str, line_no: usize) -> Result<TomlValue, TomlError> {
    let value = value.trim();
    if value.starts_with('"') {
        parse_string(value, line_no).map(TomlValue::String)
    } else if value.starts_with('[') {
        parse_array(value, line_no)
    } else if value == "true" {
        Ok(TomlValue::Boolean(true))
    } else if value == "false" {
        Ok(TomlValue::Boolean(false))
    } else {
        value
            .parse::<i64>()
            .map(TomlValue::Integer)
            .map_err(|_| TomlError::new(line_no, format!("unsupported TOML value `{value}`")))
    }
}

fn parse_array(value: &str, line_no: usize) -> Result<TomlValue, TomlError> {
    let value = value.trim();
    if !value.ends_with(']') {
        return Err(TomlError::new(line_no, "unterminated array"));
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(TomlValue::Array(Vec::new()));
    }

    let mut values = Vec::new();
    let mut rest = inner;
    while !rest.trim().is_empty() {
        rest = rest.trim_start();
        if !rest.starts_with('"') {
            return Err(TomlError::new(line_no, "only string arrays are supported"));
        }
        let (item, after) = parse_string_prefix(rest, line_no)?;
        values.push(TomlValue::String(item));
        rest = after.trim_start();
        if let Some(after_comma) = rest.strip_prefix(',') {
            rest = after_comma;
        } else if !rest.is_empty() {
            return Err(TomlError::new(line_no, "expected comma in array"));
        }
    }
    Ok(TomlValue::Array(values))
}

fn parse_string(value: &str, line_no: usize) -> Result<String, TomlError> {
    let (parsed, rest) = parse_string_prefix(value, line_no)?;
    if !rest.trim().is_empty() {
        return Err(TomlError::new(line_no, "trailing data after string"));
    }
    Ok(parsed)
}

fn parse_string_prefix(value: &str, line_no: usize) -> Result<(String, &str), TomlError> {
    let mut out = String::new();
    let mut escaped = false;
    for (idx, ch) in value.char_indices().skip(1) {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => {
                    return Err(TomlError::new(
                        line_no,
                        format!("unsupported escape sequence `\\{other}`"),
                    ));
                }
            }
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Ok((out, &value[idx + 1..])),
            other => out.push(other),
        }
    }
    Err(TomlError::new(line_no, "unterminated string"))
}

fn strip_comment(line: &str) -> &str {
    let mut escaped = false;
    let mut in_string = false;
    for (idx, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..idx],
            _ => {}
        }
    }
    line
}
