use cu::pre::*;

use crate::hmgr::config::template::{ConfigSection, ConfigTemplate};

/// Get a toml representation of leaf key values, empty tables are ignored
/// return (number of pairs, serialized toml)
pub fn serialize_leaf_key_values(value: &toml::Table) -> (usize, String) {
    let mut out = String::new();
    let mut count = 0;
    let mut stack: Vec<(Vec<&str>, &toml::Value)> = vec![];
    for (next_key, value) in value.iter().rev() {
        stack.push((vec![next_key], value));
    }
    while let Some((key, value)) = stack.pop() {
        if let toml::Value::Table(map) = value {
            for (next_key, value) in map.iter().rev() {
                let mut key = key.clone();
                key.push(next_key);
                stack.push((key, value));
            }
            continue;
        }
        if key.is_empty() {
            continue;
        }
        write_ident(&key, &mut out);
        out.push_str(" = ");
        write_value(&value, 0, &mut out);
        out.push('\n');
        count += 1;
    }

    (count, out)
}
/// Serialize a config template into the default configuration file
pub fn serialize_config_template(template: &ConfigTemplate, version: usize) -> String {
    serialize_config(template, version, &mut Default::default())
}

/// Serialize a config object. The reference values will be removed from the input value.
/// The remaining values are unused objects in the config
pub fn serialize_config(
    template: &ConfigTemplate,
    version: usize,
    value: &mut toml::Table,
) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "### version: {version}");
    let _ = writeln!(
        out,
        "### ^ Used for automatic migration, DO NOT EDIT manually"
    );

    // serialize each section, then join with empty line in between
    let mut iter = template.sections.iter();
    let Some(first) = iter.next() else {
        return out;
    };
    serialize_section(first, value, &mut out);
    for section in iter {
        out.push('\n');
        serialize_section(section, value, &mut out);
    }

    out
}
fn serialize_section(section: &ConfigSection, value: &mut toml::Table, out: &mut String) {
    use std::fmt::Write as _;

    for line in section.comment.lines() {
        let _ = writeln!(out, "# {}", line.trim_end());
    }
    let Some(section) = &section.content else {
        return;
    };
    let section_key = &section.key;
    if section_key.is_empty() {
        out.push_str("# ----------\n");
    } else {
        out.push('[');
        write_ident(section_key, out);
        out.push_str("]\n");
    }
    for entry in &section.children {
        for line in entry.comment.lines() {
            let _ = writeln!(out, "# {}", line.trim_end());
        }
        let Some(entry) = &entry.content else {
            continue;
        };
        let entry_key = &entry.key;
        write_ident(entry_key, out);
        out.push_str(" = ");
        match extract_value(section_key, entry_key, value) {
            None => {
                // use default value from template
                write_value(&entry.value, 0, out);
            }
            Some(value) => {
                write_value(&value, 0, out);
            }
        }
        out.push('\n');
    }
}

fn extract_value(
    section_key: &[String],
    entry_key: &[String],
    value: &mut toml::Table,
) -> Option<toml::Value> {
    let final_key = entry_key.last()?;

    let mut section_value = value;
    for path in section_key {
        let next = section_value.get_mut(path)?;
        section_value = next.as_table_mut()?;
    }
    let mut parent = section_value;
    for i in 0..entry_key.len() - 1 {
        let next = parent.get_mut(&entry_key[i])?;
        parent = next.as_table_mut()?;
    }

    parent.remove(final_key)
}

fn write_value(value: &toml::Value, indent: usize, out: &mut String) {
    use std::fmt::Write as _;
    const INDENT: usize = 4;
    const MAX_WIDTH: usize = 60;

    let current_len = out.len();
    value
        .serialize(toml::ser::ValueSerializer::new(out))
        .expect("should serialize toml::Value");
    if out.len() - current_len <= MAX_WIDTH {
        return;
    }
    // pretty serializer
    match value {
        toml::Value::Array(values) => {
            out.truncate(current_len);
            out.push_str("[\n");
            for value in values {
                let inner_indent = indent + INDENT;
                let _ = write!(out, "{:inner_indent$}", "");
                write_value(value, inner_indent, out);
                out.push_str(",\n");
            }
            let _ = write!(out, "{:indent$}]", "");
        }
        toml::Value::Table(map) => {
            out.truncate(current_len);
            out.push_str("{\n");
            for (key, value) in map {
                let inner_indent = indent + INDENT;
                let _ = write!(out, "{:inner_indent$}", "");
                write_one_ident(key, out);
                out.push_str(" = ");
                write_value(value, inner_indent, out);
                out.push_str(",\n");
            }
            let _ = write!(out, "{:indent$}}}", "");
        }
        _ => {
            // nothing we can do about value being too long
            return;
        }
    }
}

fn write_ident(ident: &[impl AsRef<str>], out: &mut String) {
    let mut iter = ident.iter();
    let Some(first) = iter.next() else {
        return;
    };
    write_one_ident(first.as_ref(), out);
    for part in iter {
        out.push('.');
        write_one_ident(part.as_ref(), out);
    }
}
fn write_one_ident(ident: &str, out: &mut String) {
    if ident
        .chars()
        .all(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        out.push_str(ident);
        return;
    }
    if !ident.contains('\'') {
        out.push('\'');
        out.push_str(ident);
        out.push('\'');
        return;
    }
    // escape double quoted string
    // reserve upper bound. we are likely to continue pushing
    // to the buffer anyway, so having extra memory is OK
    out.reserve(ident.len() * 6);
    out.push('"');
    for c in ident.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{0000}'..='\u{001f}' => out.push_str(&format!("\\u{:04x}", c as u8)),
            '\u{007f}' => out.push_str("\\u007f"),
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn peek_version(toml: &str) -> Option<usize> {
    let line = toml.lines().next()?;
    let version = line.strip_prefix("### version: ")?;
    cu::parse::<usize>(version).ok()
}

#[cfg(test)]
mod test {
    use crate::hmgr::config::{ConfigEntry, ConfigEntryContent, ConfigSectionContent};

    use super::*;

    #[test]
    fn simple_parse() {
        let config_file = r##"
[[section]]
comment = """
This is a test leading comment
"""

[[section]]
comment = "test_1"
key = []
children = [
    {
        comment = "foo",
        key = ["bar"],
        value = [],
    },
    {
        comment = "last"
    },
]
[[section]]
comment = "test_2"
key = ["aaa"]
children = [
    {
        comment = "foob",
        key = ["barb", "foo"],
        value = 0,
    },
]
        "##;
        let template: ConfigTemplate = toml::parse(config_file).expect("failed to parse");
        assert_eq!(
            template.sections,
            vec![
                ConfigSection {
                    comment: "This is a test leading comment".to_string(),
                    content: None
                },
                ConfigSection {
                    comment: "test_1".to_string(),
                    content: Some(ConfigSectionContent {
                        key: vec![],
                        children: vec![
                            ConfigEntry {
                                comment: "foo".to_string(),
                                content: Some(ConfigEntryContent {
                                    key: vec!["bar".to_string()],
                                    value: toml::Value::Array(vec![])
                                })
                            },
                            ConfigEntry {
                                comment: "last".to_string(),
                                content: None
                            },
                        ]
                    })
                },
                ConfigSection {
                    comment: "test_2".to_string(),
                    content: Some(ConfigSectionContent {
                        key: vec!["aaa".to_string()],
                        children: vec![ConfigEntry {
                            comment: "foob".to_string(),
                            content: Some(ConfigEntryContent {
                                key: vec!["barb".to_string(), "foo".to_string()],
                                value: toml::Value::Integer(0)
                            })
                        },]
                    })
                },
            ]
        );

        let mut value = toml! {
            bar = [ 1, 2, 3]
            aaa.barb.foo = "hello"
            this.is.unused = { key = "hehehe" }
        };

        assert_eq!(
            serialize_config(&template, 123, &mut value).trim(),
            r##"
### version: 123
### ^ Used for automatic migration, DO NOT EDIT manually
# This is a test leading comment

# test_1
# ----------
# foo
bar = [1, 2, 3]
# last

# test_2
[aaa]
# foob
barb.foo = "hello"
        "##
            .trim()
        );
        let (count, remaining) = serialize_leaf_key_values(&value);
        assert_eq!(count, 1);
        assert_eq!(remaining, "this.is.unused.key = \"hehehe\"\n");
    }

    #[test]
    pub fn pretty_value() {
        let value = toml::Value::Table(toml! {
            database = {
                ip = "192.168.1.1",
                port = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port2 = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port3 = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port4 = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010, 8011],
                connection_max = 5000,
                enabled = false,
            }
            database2 = {
                ip = "192.168.1.1",
            }
        });
        let mut out = String::new();
        write_value(&value, 8, &mut out);
        let expected = r##"
        {
            database = {
                ip = "192.168.1.1",
                port = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port2 = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port3 = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010],
                port4 = [
                    8001,
                    8002,
                    8003,
                    8004,
                    8005,
                    8006,
                    8007,
                    8008,
                    8009,
                    8010,
                    8011,
                ],
                connection_max = 5000,
                enabled = false,
            },
            database2 = { ip = "192.168.1.1" },
        }
        "##;
        assert_eq!(out.trim(), expected.trim());
    }
}
