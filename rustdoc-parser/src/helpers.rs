use std::collections::BTreeMap;

/////////////////////////////////////////////////////////////////////////////
// Utilitarian functions
/////////////////////////////////////////////////////////////////////////////

/// Extract the content of a sepcific element and returns its full content as Fragments.
pub fn extract(name: &str, data: &str) -> HtmlElement {
    let closed_tag = format!("<{}>", name);
    let opened_tag = format!("<{}", name);
    let untag = closed_tag.replace('<', "</");
    let tail = if let Some((_, tail)) = data.split_once(&closed_tag) {
        format!("{}{}", closed_tag, tail)
    } else if let Some((_, tail)) = data.split_once(&opened_tag) {
        format!("{}{}", opened_tag, tail)
    } else {
        String::from(data)
    };
    let mut tag = tail
        .split_inclusive(&untag)
        .next()
        .expect("extract_tag failed to get rid of the trailing data");
    
    HtmlElement::build(&mut tag)
}

/// Turns escaped character back into plain characters.
pub fn un_escape(raw: &str) -> String {
    raw.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&#x27;", "'")
}

/// HtmlElement is a helper type used for gathering every piece of information about an HTML element.
///
/// It includes :
/// - a kind, which is the main tag, such as "div", "a", ...
/// - optional additional attributes, such as "id", "class", ...
/// - optional raw content
/// - optional inner elements

#[derive(Clone, Debug, PartialEq)]
pub struct HtmlElement {
    pub kind: String,
    attributes: BTreeMap<String, String>,
    pub content: Vec<Fragment>,
    pub inner_elements: Vec<HtmlElement>,
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////

impl HtmlElement {
    /// Build a new HtmlElement out of an html element and all of its inner elements.
    ///
    /// Makes use of the :
    /// - Tag type to assess the full identity of an opening, closing or void tag,
    /// - Fragment type to segment its content into formatted, or not, entities.
    ///
    /// # Panics
    ///
    /// Panics if the HtmlElement cannot be built all the way through
    pub fn build(html: &mut &str) -> Self {
        let main_tag_end = html
            .find('>')
            .expect("Html::build() failed : can't compute main_tag_end");
        let main_tag = Tag::build(&html[1..main_tag_end]);
        let attributes = main_tag.attributes;
        let mut content: Vec<Fragment> = Vec::new();
        let mut inner_elements: Vec<HtmlElement> = Vec::new();

        *html = html[main_tag_end + 1..].trim();

        loop {
            if html.starts_with('<') {
                let current_tag_end = html
                    .find('>')
                    .expect("Html::build() failed : can't compute current_current_tag_end");
                let current_tag = Tag::build(&html[1..current_tag_end]);
                match current_tag.kind {
                    TagKind::Void => {
                        inner_elements.push(HtmlElement {
                            kind: current_tag.name,
                            attributes: current_tag.attributes,
                            content: Vec::new(),
                            inner_elements: Vec::new(),
                        });

                        *html = html[current_tag_end + 1..].trim();
                    }
                    TagKind::Opening => inner_elements.push(HtmlElement::build(html)),
                    TagKind::Closing => {
                        *html = html[current_tag_end + 1..].trim();

                        return HtmlElement {
                            kind: main_tag.name,
                            attributes,
                            content,
                            inner_elements,
                        };
                    }
                }
            } else {
                if html.is_empty() {
                    return HtmlElement {
                        kind: main_tag.name,
                        attributes,
                        content,
                        inner_elements,
                    };
                }
                let next_element_start = html
                    .find('<')
                    .expect("Html::build() failed : can't compute the next element");
                content.push(Fragment::Raw(String::from(&html[..next_element_start])));

                *html = &html[next_element_start..];
            }
        }
    }

    /// Returns the inner block's full content as fragments through zipping recursively.
    pub fn zip_content(&mut self) -> Vec<Fragment> {
        if !self.inner_elements.is_empty() && self.inner_elements[0].kind == "strong" {
            return Vec::from([
                Fragment::Bold(Vec::from([self.inner_elements[0].content[0].clone()])),
                self.content[0].clone(),
            ]);
        }
        let content = &self.content;
        let inner_elements_content = self
            .inner_elements
            .iter_mut()
            .filter(|inner_element| inner_element.kind != "div")
            .map(|inner_element| {
                if let Some(r#type) = inner_element.attributes.get("class") {
                    let name = inner_element.content[0].clone();
                    match r#type.as_str() {
                        "trait" => Fragment::Colored(Box::new(name), Color::Trait),
                        "struct" => Fragment::Colored(Box::new(name), Color::Struct),
                        "enum" => Fragment::Colored(Box::new(name), Color::Enum),
                        "macro" => Fragment::Colored(Box::new(name), Color::Macro),
                        "primitive" => Fragment::Colored(Box::new(name), Color::Primitive),
                        "fn" => Fragment::Colored(Box::new(name), Color::Method),
                        "associatedtype" => {
                            Fragment::Colored(Box::new(name), Color::AssociatedType)
                        }
                        _ => name,
                    }
                } else if &inner_element.kind == "code" {
                    Fragment::Code(Box::new(inner_element.content[0].clone()))
                } else if !&inner_element.inner_elements.is_empty() {
                    match inner_element.inner_elements[0].kind.as_str() {
                        "code" => Fragment::Code(Box::new(
                            inner_element.inner_elements[0].content[0].clone(),
                        )),
                        "a" | _ => inner_element.inner_elements[0].content[0].clone(),
                    }
                } else {
                    inner_element.content[0].clone()
                }
            });
        let mut zipped_content = content.iter().zip(inner_elements_content).fold(
            Vec::new(),
            |mut zipped_content, (raw_content, inner_content)| {
                zipped_content.push(raw_content.clone());
                zipped_content.push(inner_content);

                zipped_content
            },
        );
        if content.len() > self.inner_elements.len()
            && let Some(last_piece_of_raw_content) = content.clone().pop()
        {
            zipped_content.push(last_piece_of_raw_content);
        }
        if let Some(mut last_inner_element) = self.inner_elements.pop()
            && let Some(class) = last_inner_element.attributes.get("class")
            && class == "where"
        {
            zipped_content.append(&mut last_inner_element.zip_content());
        }

        zipped_content
            .iter_mut()
            .map(|fragment| fragment.clone().un_escape_content())
            .collect::<Vec<Fragment>>()
    }

    /// Returns the content of a CodeBlock as a single Raw Fragment.
    pub fn zip_code(&mut self) -> Box<Fragment> {
        let spans = self
            .inner_elements
            .iter()
            .map(|span| span.content[0].clone());
        let last_span = spans.clone().last().unwrap();
        let content = &self.content;

        let mut zipped_content =
            spans
                .zip(content)
                .fold(Vec::new(), |mut zipped_content, (span, content)| {
                    zipped_content.push(span);
                    zipped_content.push(content.clone());

                    zipped_content
                });
        if self.inner_elements.len() > content.len() {
            zipped_content.push(last_span);
        }

        Box::new(zipped_content
            .into_iter()
            .reduce(|previous, next| {
                let previous_content = un_escape(previous.raw_content());
                let next_content = un_escape(next.raw_content());
                Fragment::Raw(format!("{}{}", previous_content, next_content))
            })
            .unwrap()
        )
    }
}

/// The full content of a tag.
#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
    kind: TagKind,
    attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
enum TagKind {
    Closing,
    Opening,
    Void,
}

impl Tag {
    /// Builds a new Tag.
    pub fn build(data: &str) -> Self {
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

/// Represents a single piece of information, be it a full sentence or a mere word.
///
/// Shall help bring the important parts out when rendering the documentation in the terminal.
#[derive(Clone, Debug, PartialEq)]
pub enum Fragment {
    Bold(Vec<Fragment>),
    Code(Box<Fragment>),
    CodeBlock(Box<Fragment>),
    Colored(Box<Fragment>, Color),
    Raw(String),
}

impl Fragment {
    /// Helps retrieve the inner String out of a Fragment::Raw.
    pub fn raw_content(&self) -> &str {
        if let Fragment::Raw(raw_content) = self {
            raw_content
        } else {
            panic!("Tried to call raw_content on an invalid Fragment")
        }
    }

    /// Uses the un_escape function to un_escape the full inner content of a Fragment.
    pub fn un_escape_content(self) -> Self {

        match self {
            Fragment::Bold(fragments) => Fragment::Bold(
                fragments
                    .into_iter()
                    .map(|f| f.un_escape_content())
                    .collect::<Vec<Fragment>>(),
            ),
            Fragment::Code(boxed_fragment) => {
                Fragment::Code(Box::new(boxed_fragment.un_escape_content()))
            }
            Fragment::CodeBlock(boxed_fragment) => {
                Fragment::CodeBlock(Box::new(boxed_fragment.un_escape_content()))
            },
            Fragment::Colored(boxed_fragment, color) => {
                Fragment::Colored(Box::new(boxed_fragment.un_escape_content()), color)
            }
            Fragment::Raw(ref raw_content) => Fragment::Raw(un_escape(raw_content)),
        }
    }
}

/// Will help in the process of highlighting content when rendering said content in the terminal.
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    AssociatedType,
    Enum,
    Macro,
    Method,
    Primitive,
    Struct,
    Trait,
}

impl Color {
    pub fn to_hex(&self) -> &str {
        match self {
            Color::AssociatedType => "#d2991d",
            Color::Enum => "#2dbfb8",
            Color::Macro => "#09bd00",
            Color::Method => "#2bab63",
            Color::Primitive => "#2dbfb8",
            Color::Struct => "#2dbfb8",
            Color::Trait => "#b78cf2",
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
