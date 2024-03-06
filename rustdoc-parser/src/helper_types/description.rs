use super::{
    fragment::Fragment,
    html_element::HtmlElement
};

/// Represents a method description, be it a couple of paragraphs, or entire subsections.
#[derive(Debug)]
pub struct Description {
    pub introduction: Vec<Fragment>,
    pub sections: Vec<DescriptionSection>,
}

impl Description {
    /// Parses an up to several lines long description.
    pub fn parse(description: &str) -> Description {
        let (_introduction, sections) = if let Some((__introduction, __sections)) = description.split_once("<h5") {
            let _sections = __sections
                .split("<h5")
                .map(|section| DescriptionSection::parse(section))
                .collect();
            (__introduction, _sections)
        } else {
            (description, Vec::new())
        };
        // introduction does NOT necessarily contains <p>s
        let introduction = _introduction
            .split_inclusive("</p>\n")
            .flat_map(|description_line| HtmlElement::extract("p", description_line).zip_content())
            .collect();

        Description {
            introduction,
            sections,
        }
    }
}

/// Represents a description's possible subsection - Examples, Panics, and what have you.
#[derive(Debug)]
pub struct DescriptionSection {
    pub name: Fragment,
    pub content: Vec<Fragment>,
}

impl DescriptionSection {
    /// Parses a description possible subsection - Examples, Panics, and what have you.
    fn parse(data: &str) -> Self {
        let (_content, mut sections) = data.split_once("</h5>").unwrap();
        let name = Fragment::Bold(Vec::from([HtmlElement::extract("a", _content).content[0].clone()]));
        let mut content = Vec::new();
        loop {
            match sections.trim().chars().nth(1) {
                Some('p') => {
                    let (head, tail) = sections.split_once("</p>").unwrap();
                    content.push(
                        HtmlElement::parse(&mut format!("{}</p>", head).as_str()).zip_content(),
                    );
                    sections = tail;
                }
                Some('d') => {
                    let (head, tail) = sections.split_once("</div>").unwrap();
                    content.push(Vec::from([Fragment::CodeBlock(
                        HtmlElement::extract("code", head).zip_code(),
                    )]));
                    sections = tail;
                }
                Some(_) | None => break,
            }
        }

        DescriptionSection {
            name,
            content: content.into_iter().flatten().collect(),
        }
    }
}

