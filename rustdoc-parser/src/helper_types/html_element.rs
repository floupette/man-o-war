use std::collections::BTreeMap;

use super::{
    fragment::{
        Color,
        Fragment
        },
    tag::{
        Tag,
        TagKind
        }
};

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

impl HtmlElement {
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

        HtmlElement::parse(&mut tag)
        }
    /// Build a new HtmlElement out of an html element and all of its inner elements.
    ///
    /// Makes use of the :
    /// - Tag type to assess the full identity of an opening, closing or void tag,
    /// - Fragment type to segment its content into formatted, or not, entities.
    ///
    /// # Panics
    ///
    /// Panics if the HtmlElement cannot be built all the way through
    pub fn parse(html: &mut &str) -> Self {
        let main_tag_end = html
            .find('>')
            .expect("Html::build() failed : can't compute main_tag_end");
        let main_tag = Tag::parse(&html[1..main_tag_end]);
        let attributes = main_tag.attributes;
        let mut content: Vec<Fragment> = Vec::new();
        let mut inner_elements: Vec<HtmlElement> = Vec::new();

        *html = html[main_tag_end + 1..].trim();

        loop {
            if html.starts_with('<') {
                let current_tag_end = html
                    .find('>')
                    .expect("Html::build() failed : can't compute current_current_tag_end");
                let current_tag = Tag::parse(&html[1..current_tag_end]);
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
                    TagKind::Opening => inner_elements.push(HtmlElement::parse(html)),
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
                        // a containing code
                        "code" => Fragment::Code(Box::new(
                            inner_element.inner_elements[0].content[0].clone(),
                        )),
                        // em containing a
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
        // where clauses
        if let Some(mut last_inner_element) = self.inner_elements.pop()
            && let Some(class) = last_inner_element.attributes.get("class")
            && class == "where"
        {
            // adding trailing content before the where clause
            if content.len() > self.inner_elements.len()
                && let Some(last_piece_of_raw_content) = content.clone().pop()
            {
                zipped_content.push(last_piece_of_raw_content);
            }
            zipped_content.append(&mut last_inner_element.zip_content());
            return zipped_content
                .iter_mut()
                .map(|fragment| fragment.clone().un_escape_content())
                .collect::<Vec<Fragment>>();
        }
        // zipping vecs of unequal sizes leaves content behind
        if content.len() > self.inner_elements.len()
            && let Some(last_piece_of_raw_content) = content.clone().pop()
        {
            zipped_content.push(last_piece_of_raw_content);
        }

        zipped_content
            .iter_mut()
            .map(|fragment| fragment.un_escape_content())
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

        Box::new(
            zipped_content
                .into_iter()
                .reduce(|previous, next| {
                    Fragment::Raw(format!("{}{}",
                        previous.un_escape_content().raw_content(),
                        next.un_escape_content().raw_content()
                    ))
                })
                .unwrap(),
        )
    }
}
