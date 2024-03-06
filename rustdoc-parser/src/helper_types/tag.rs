use std::collections::BTreeMap;

/// The full content of a tag.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub name: String,
    pub kind: TagKind,
    pub attributes: BTreeMap<String, String>,
}

/// Any of a tag's kind.
#[derive(Clone, Debug, PartialEq)]
pub enum TagKind {
    Closing,
    Opening,
    Void,
}

impl Tag {
    /// Parses a new Tag.
    pub fn parse(data: &str) -> Self {
        let name;
        let attributes;
        if let Some((tag_name, content)) = data.split_once(' ') {
            name = String::from(tag_name);
            let mut attribs = Vec::new();
            let mut values = Vec::new();
            content
                .split_terminator('"')
                .enumerate()
                .for_each(|(index, split)| {
                    if index & 1 == 0 {
                        attribs.push(split.trim_matches(|c| c == ' ' || c == '='));
                    } else {
                        values.push(split.trim());
                    }
                });
            attributes = BTreeMap::from_iter(
                attribs
                    .iter()
                    .zip(values)
                    .map(|(key, value)| (String::from(*key), String::from(value))),
            );
        } else {
            name = String::from(data);
            attributes = BTreeMap::new();
        }
        match name.as_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "param" | "source" | "track" | "wbr" => Tag {
                name,
                kind: TagKind::Void,
                attributes,
            },
            _ => {
                let kind = if name.starts_with('/') {
                    TagKind::Closing
                } else {
                    TagKind::Opening
                };

                Tag {
                    name,
                    kind,
                    attributes,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tag, TagKind};
    use std::collections::BTreeMap;
    #[test]
    fn tag_build() {
        let data =
            r#"a href="emoclew\08:1.0.0.721\\:ptth" type="url Home" class="Url" open>!emocleW"#;
        let attributes = BTreeMap::from([
            (
                String::from("href"),
                String::from(r#"emoclew\08:1.0.0.721\\:ptth"#),
            ),
            (String::from("type"), String::from("url Home")),
            (String::from("class"), String::from("Url")),
        ]);
        let result = Tag {
            name: String::from("a"),
            kind: TagKind::Opening,
            attributes,
        };
        assert_eq!(result, Tag::build(data));
    }
}
