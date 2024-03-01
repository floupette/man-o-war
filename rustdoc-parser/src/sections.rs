use super::{
    helpers::{
		extract,
		Fragment,
		HtmlElement
		},
	traits::Body,
};

/// Represents a single auto trait implementation.
#[derive(Debug)]
pub struct AutoTrait(pub Vec<Fragment>);
impl Body for AutoTrait {}

/// Specifically parses the Auto Trait Implementations section.
pub fn parse_auto_traits(data: &str) -> Vec<Box<dyn Body>> {
    data.split("</section>")
        .filter(|i| *i != "</div>")
        .map(|implementation| -> Box<dyn Body> {
            Box::new(AutoTrait(Vec::from([Fragment::Bold(extract(
                "h3",
                implementation,
            ).zip_content())])))
        })
        .collect()
}

/// Represents a single implementation for one specific type.
#[derive(Debug)]
pub struct Implementation {
    pub name: Vec<Fragment>,
    pub methods: Vec<Method>,
}
impl Body for Implementation {}

/// Specifically parses the Implementations section.
pub fn parse_implementations(data: &str) -> Vec<Box<dyn Body>> {
    data.split("<details class=\"toggle implementors-toggle\" open>")
        .skip(1)
        .map(|imp| -> Box<dyn Body> {
            let (unfiltered_content, _methods) = imp.split_once("</summary>").unwrap();
            let implementation = Vec::from([Fragment::Bold(extract("h3", unfiltered_content).zip_content())]);
            let methods = _methods
                .trim_end_matches("</div></details>")
                .split("<details class=\"toggle method-toggle\" open>")
                .skip(1)
                .map(parse_method)
                .collect();
            Box::new(Implementation {
                implementation,
                methods,
            })
        })
        .collect()
}

/// Represents one of an Implementation's possible methods.
#[derive(Debug)]
pub struct Method {
    pub signature: Vec<Fragment>,
    pub description: Description,
}

/// Parses the one method into its signature and description.
fn parse_method(method: &str) -> Method {
    Method {
        signature: extract("h4", method).zip_content(),
        description: parse_description(method),
    }
}

/// Represents a method description, be it a couple of paragraphs, or entire subsections.
#[derive(Debug)]
pub struct Description {
    pub introduction: Vec<Fragment>,
    pub sections: Vec<DescriptionSection>,
}

/// Represents a method's introduction's possible subsection - Examples, Panics, and what have you.
#[derive(Debug)]
struct DescriptionSection {
    pub name: Fragment,
    pub content: Vec<Fragment>,
}

/// Parses a method's description.
fn parse_description(description: &str) -> Description {
    let mut sections = Vec::new();
    let mut _introduction = "";
    if let Some((__introduction, _sections)) = description.split_once("<h5") {
        _introduction = __introduction;
        sections = _sections
            .split("<h5")
            .map(|_section| {
                let (_content, mut __sections) = _section.split_once("</h5>").unwrap();
                let section = Fragment::Bold(Vec::from([Fragment::Raw(String::from(
                    _content.split_once("</a>").unwrap().1,
                ))]));
                let mut content = Vec::new();
                loop {
                    match __sections.trim().chars().nth(1) {
                        Some('p') => {
                            let (head, tail) = __sections.split_once("</p>").unwrap();
                            content.push(
                                HtmlElement::build(&mut format!("{}</p>", head).as_str())
                                    .zip_content(),
                            );
                            __sections = tail;
                        }
                        Some('d') => {
                            let (head, tail) = __sections.split_once("</div>").unwrap();
                            content.push(Vec::from([Fragment::CodeBlock(extract("code", head).zip_code())]));
                            __sections = tail;
                        }
                        Some(_) | None => break,
                    }
                }

                DescriptionSection {
                    section,
                    content: content.iter().flatten().cloned().collect(),
                }
            })
            .collect::<Vec<DescriptionSection>>()
    } else {
        _introduction = description;
    };
    let introduction = _introduction
        .split_inclusive("</p>\n")
        .flat_map(|description_line| extract("p", description_line.trim_end()).zip_content())
        .collect();

    Description {
        introduction,
        sections,
    }
}

/// Represents a single variant of an Enum.
#[derive(Debug)]
pub struct Variant {
    pub name: Fragment,
    pub description: Vec<Fragment>,
}
impl Body for Variant {}

/// Specifically parses the Variants section.
pub fn parse_variants(data: &str) -> Vec<Box<dyn Body>> {
    data.split("</a>")
        .skip(1)
        .map(|variant| -> Box<dyn Body> {
            Box::new(Variant {
                name: Fragment::Bold(extract("h3", variant).zip_content()),
                description: extract("p", variant).zip_content(),
            })
        })
        .collect()
}
