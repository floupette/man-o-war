//! # Rustdoc-parser
//!
//! First, the documentation generated by cargo must be parsed.
//!
//! # Parsing
//!
//! Through extensive splitting and trimming, we make our way through the HTML source.
//!
//! With the help of the Html type, the important elements are brought to a then agnostic life.
//!
//! # Agnosticity
//!
//! Content is then patched up when the raw text is zipped with its subsections', such as `<a>`s and `<code>`s.
//!
//! ## Sidebar
//!
//! Parsing the Sidebar is rather straightforward and the element soon built up.
//!
//! ## Main Content
//!
//! The main content, though, implies going through differently constructed sections.
//!
//! Auto Trait Implementations, Immplementation and Variants are a-OK.
//!
//! On to Trait Implementations.

#![allow(dead_code, unused_assignments, unused_variables)]
#![feature(let_chains)]

pub mod errors;
pub mod helpers;
pub mod main_content;
pub mod sections;
pub mod sidebar;
pub mod traits;

use crate::{
    errors::Herr,
    helpers::Fragment,
    main_content::MainContent,
    sections::Description,
    sidebar::Sidebar
};

#[derive(Debug)]
pub struct Page {
    entry: Vec<Fragment>,
    sidebar: Sidebar,
    introduction: Description,
    main_content: MainContent,
}

/////////////////////////////////////////////////////////////////////////////
// main function
/////////////////////////////////////////////////////////////////////////////

/// Converts an HTML document into a Page.
pub fn process_html(html: &str) -> Result<(), Herr> {
    let mut sidebar: Sidebar = Sidebar(Vec::new());
    let mut main_content = MainContent(Vec::new());
    html.split_once("<section>")
        .and_then(|(_head, body)| body.split_once("</section>"))
        .and_then(|(sidebar_content, main)| {
            sidebar = Sidebar::build(sidebar_content);
            main.split_once("<section id=\"main-content\" class=\"content\">")
        })
        .and_then(|(_, main_c)| main_c.split_once("<script"))
        .and_then(|(main_cont, _)| main_cont.split_once("</details>"))
        .map(|(_introduction_content, section_block)| {
            main_content = MainContent::build(section_block);
            dbg!(&main_content);
        })
        .ok_or(Herr::Parsing("Can't process that page, sir"))
}
