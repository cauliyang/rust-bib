use log::info;
use scraper::{Html, Selector};

// view-source:https://pubmed.ncbi.nlm.nih.gov/33441414/

pub fn fetch_citation_key(key: &str, html: &Html) -> Option<String> {
    let selector = Selector::parse(&*format!(r#"meta[name="{}"]"#, key)).unwrap();
    info!("key: {}", key);

    let element = html.select(&selector).into_iter().next();

    element?;

    element
        .unwrap()
        .value()
        .attr("content")
        .map(|x| x.to_string())
}

pub fn fetch_title(html: &Html) -> Option<String> {
    let key = "citation_title";
    fetch_citation_key(key, html)
}

pub fn fetch_author(html: &Html) -> Option<String> {
    let key = "citation_authors";
    fetch_citation_key(key, html).map(|s| s.trim_end_matches(';').to_string())
}

pub fn fetch_year(html: &Html) -> Option<String> {
    let key = "citation_date";
    let text = fetch_citation_key(key, html).unwrap();
    info!("year: {}", text);
    if text.contains('/') {
        let year = text.split('/').last().unwrap();
        Some(year.to_string())
    } else {
        text.split(' ').next().map(|s| s.to_string())
    }
}

pub fn fetch_journal(html: &Html) -> Option<String> {
    let key = "citation_journal_title";
    fetch_citation_key(key, html)
}

pub fn fetch_volume(html: &Html) -> Option<String> {
    let key = "citation_volume";
    fetch_citation_key(key, html)
}

pub fn fetch_publisher(html: &Html) -> Option<String> {
    #[allow(unused)]
    let key = "citation_publisher";

    let selector = Selector::parse(r#"p[class="copyright"]"#).unwrap();

    let element = html
        .select(&selector)
        .into_iter()
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join(" ");

    let element = element
        .trim()
        .split("by")
        .last()
        .unwrap()
        .trim()
        .to_string();

    info!("element: {}", element);

    Some(element)
}

pub fn fetch_doi(html: &Html) -> Option<String> {
    let key = "citation_doi";
    fetch_citation_key(key, html)
}

pub fn fetch_number(html: &Html) -> Option<String> {
    let key = "citation_issue";
    fetch_citation_key(key, html)
}

fn check_page(page: &str) -> bool {
    page.contains('-')
}

pub fn fetch_page(html: &Html) -> Option<String> {
    // <span class="cit">2021 Mar;31(3):448-460.</span>
    let selector = Selector::parse(r#"span[class="cit"]"#).unwrap();
    let text = html
        .select(&selector)
        .into_iter()
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        return None;
    }

    let page = text
        .split(':')
        .last()
        .unwrap()
        .trim_end_matches('.')
        .to_string();
    info!("page: {}", page);

    if check_page(&page) {
        Some(page)
    } else {
        None
    }
}
