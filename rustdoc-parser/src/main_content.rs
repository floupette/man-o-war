use crate::{
    helpers::{
        Fragment,
        HtmlElement
        },
    sections::{
        parse_auto_traits,
        parse_implementations,
        parse_variants
    },
    traits::Body
};

/////////////////////////////////////////////////////////////////////////////
// Main Content
/////////////////////////////////////////////////////////////////////////////

/// The main content of a single page, from the introduction to the last of its implementation blocks.
#[derive(Debug)]
pub struct MainContent(pub Vec<Section>);

// /// A part of a Section.
// #[derive(Clone, Debug, Default)]
// pub struct Subsection {
//     pub content: Vec<Fragment>,
//     pub inner_subsections: Vec<Subsection>,
// }

impl MainContent {
    /// Builds the main content ouf of its sections.
    pub fn build(raw: &str) -> Self {
        let sections = raw
            .split("<h2")
            .skip(1)
            .fold(Vec::new(), |mut sections, str| {
                sections.push(format!("<h2{}", str));
                sections
            });

        MainContent(sections
            .iter()
            .map(|section| {
                let mut split = section.split_inclusive("</h2>").collect::<Vec<&str>>();
                let name = String::from(HtmlElement::build(&mut split[0]).content[0].raw_content());
                // LATERZ
                // let (_subsections, _) = split[1]
                //     .split_once("\n</div></details></div></details>")
                //     .unwrap();
                let body = match name.as_str() {
                    "Variants" => parse_variants(split[1]), // OK
                    "Auto Trait Implementations" => parse_auto_traits(split[1]), // OK
                    "Implementations" => parse_implementations(split[1]), // OK
                    // "Trait Implementations" => parse_implementations(split[1]), // NOPE
                    // Doesn't panic, but information is missing
                    // IntoIterator => Notable Traits
                    // FromIterator => Examples out of an Examples subsection
                    // type: Item, ...
                    // check for nightly/unstable in a span right after summary <= might be the case elsewhere in the library
                    _ => Vec::new(),
                };
                Section {
                    name: Fragment::Bold(Vec::from([Fragment::Raw(name)])),
                    body,
                }
        })
        .collect())
    }
}

/// A segment of the main contentt.
#[derive(Debug)]
pub struct Section {
    pub name: Fragment,
    pub body: Vec<Box<dyn Body>>,
}

impl Section {
    pub fn build(name: String, body: Vec<Box<dyn Body>>) -> Self {
        Section {
            name: Fragment::Bold(Vec::from([Fragment::Raw(name)])),
            body,
        }
    }
}
