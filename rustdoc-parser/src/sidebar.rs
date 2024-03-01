use crate::helpers::{
    HtmlElement,
    un_escape
};

/////////////////////////////////////////////////////////////////////////////
// Sidebar
/////////////////////////////////////////////////////////////////////////////

/// Parses the full Sidebar of a page in a rather neat fashion.
#[derive(Debug)]
pub struct Sidebar(pub Vec<SidebarSection>);

/// Represents a section of the Sidebar. Methods, Implementations, and such.
#[derive(Debug)]
pub struct SidebarSection {
    name: String,
    items: Vec<String>,
}

impl Sidebar {
    /// Builds a new Sidebar.
    pub fn build(content: &str) -> Self {
        Sidebar(
            content
                .split_inclusive("</ul>")
                .map(|s| {
                    let mut split = s.split_inclusive("</h3>").collect::<Vec<&str>>();
                    let heading = HtmlElement::build(&mut split[0]);
                    let inner_a = &heading.inner_elements[0];
                    let name = String::from(inner_a.content[0].raw_content());
                    let items = HtmlElement::build(&mut split[1])
                        .inner_elements
                        .iter()
                        .map(|inner_element| {
                            let first_inner_element_content =
                                &inner_element.inner_elements[0].content[0].raw_content();
                            un_escape(first_inner_element_content)
                        })
                        .collect::<Vec<String>>();

                    SidebarSection { name, items }
                })
                .collect::<Vec<SidebarSection>>(),
        )
    }
}
