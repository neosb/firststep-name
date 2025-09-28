use askama::Template;
// use poem::{Response, http::StatusCode};

#[derive(Template)]
#[template(ext = "html", path = "hej.html")]
pub struct HelloTemplate<'a> {
    pub title: &'a str,
    pub name: &'a str,
    pub year: i32, // Current year
}

#[derive(Template)]
#[template(ext = "html", path = "head.html")]
pub struct HeadTemplate<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(ext = "html", path = "footer.html")]
pub struct FooterTemplate {
    pub year: i32,
}

#[derive(Template)]
#[template(ext = "html", path = "content.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub year: i32, // Current year
}

#[derive(Template)]
#[template(ext = "html", path = "script.html")]
pub struct ScriptTemplate {}

#[derive(Template)]
#[template(ext = "html", path = "style.html")]
pub struct StyleTemplate {}
