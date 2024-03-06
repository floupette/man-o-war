use crate::helper_types::html_element::HtmlElement;

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
    pub fn parse(content: &str) -> Self {
        Sidebar(
            content
                .split_inclusive("</ul>")
                .map(|s| {
                    let mut split = s.split_inclusive("</h3>").collect::<Vec<&str>>();
                    let heading = HtmlElement::parse(&mut split[0]);
                    let inner_a = &heading.inner_elements[0];
                    let name = String::from(inner_a.content[0].raw_content());
                    let items = if split.len() > 1 {
                        HtmlElement::parse(&mut split[1])
                            .inner_elements
                            .iter()
                            .map(|inner_element| {
                                if inner_element.inner_elements.is_empty() {
                                    String::new()
                                } else {
                                    String::from(inner_element.inner_elements[0].content[0].un_escape_content().raw_content())
                                }
                            })
                            .collect::<Vec<String>>()
                    } else {
                        Vec::new()
                    };

                    SidebarSection { name, items }
                })
                .collect::<Vec<SidebarSection>>(),
        )
    }
}
