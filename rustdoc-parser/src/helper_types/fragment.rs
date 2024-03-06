/// Represents a single piece of information, be it a full sentence or a mere symbol.
///
/// Shall help bring the important parts out when rendering the documentation in the terminal.
#[derive(Clone, Debug, PartialEq)]
pub enum Fragment {
    Bold(Vec<Fragment>),
    Code(Box<Fragment>),
    CodeBlock(Box<Fragment>),
    Colored(Box<Fragment>, Color),
    Raw(String)
}

impl Fragment {
    /// Helps retrieve the inner String out of a Fragment::Raw.
    pub fn raw_content(&self) -> &str {
        let Fragment::Raw(raw_content) = self else {
            panic!("Tried to call raw_content on an invalid Fragment")
        };

        raw_content
    }

    /// Uses the un_escape function to un_escape the full inner content of a Fragment.
    pub fn un_escape_content(&self) -> Self {
        match self {
            Fragment::Bold(fragments) => Fragment::Bold(
                fragments
                    .into_iter()
                    .map(|f| f.un_escape_content())
                    .collect::<Vec<Fragment>>(),
            ),
            Fragment::Code(boxed_fragment) => Fragment::Code(Box::new(boxed_fragment.un_escape_content())),
            Fragment::CodeBlock(boxed_fragment) => Fragment::CodeBlock(Box::new(boxed_fragment.un_escape_content())),
            Fragment::Colored(boxed_fragment, color) => Fragment::Colored(Box::new(boxed_fragment.un_escape_content()), color.clone()),
            Fragment::Raw(ref raw_content) => Fragment::Raw(un_escape(raw_content)),
        }
    }
}

/// Turns escaped character back into plain characters.
pub fn un_escape(raw: &str) -> String {
    raw.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&#x27;", "'")
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
