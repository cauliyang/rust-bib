#[deny(warnings)]
#[allow(unused_imports)]
use clap::Parser;
use fmt::Display;
use log::{debug, info, warn};
use scraper::{Html, Selector};
use std::fmt;
use std::fmt::Formatter;

// @article{RN109,
// author = {Domscheit, H. and Hegeman, M. A. and Carvalho, N. and Spieth, P. M.},
// title = {Molecular Dynamics of Lipopolysaccharide-Induced Lung Injury in Rodents},
// journal = {Front Physiol},
// volume = {11},
// pages = {36},
// ISSN = {1664-042X (Print)
// 1664-042X (Linking)},
// DOI = {10.3389/fphys.2020.00036},
// url = {https://www.ncbi.nlm.nih.gov/pubmed/32116752},
// year = {2020},
// type = {Journal Article}
// }

const BASE_URL: &str = "https://pubmed.ncbi.nlm.nih.gov";
const META_KEYS: [&str; 11] = [
    "description",
    "citation_title",
    "citation_authors",
    "citation_date",
    "citation_journal_title",
    "citation_pmid",
    "citation_volume",
    "citation_issue",
    "citation_publisher",
    "citation_doi",
    "citation_isbn",
];

#[derive(Debug)]
struct Bibtex {
    btype: String,
    title: String,
    authors: String,
    year: String,
    journal: String,
    volume: String,
    publisher: Option<String>,
    doi: Option<String>,
    isbn: Option<String>,
    month: Option<String>,
    pages: Option<String>,
}

impl Bibtex {
    #[allow(dead_code)]
    fn cite_key(&self) -> String {
        let mut cite_key = String::new();
        let first_author_last_name = &self.authors.split_whitespace().next().unwrap();
        cite_key.push_str(first_author_last_name);
        cite_key.push_str(&self.year.split_whitespace().next().unwrap());
        cite_key.push_str(&self.title.split_whitespace().next().unwrap());
        cite_key
    }

    #[allow(dead_code)]
    fn new(html: &Html) -> Bibtex {
        let bibtex = Bibtex {
            btype: "article".to_string(),
            title: fetch_citation_key("citation_title", html).unwrap(),
            authors: fetch_citation_key("citation_authors", html).unwrap(),
            year: fetch_citation_key("citation_date", html).unwrap(),
            journal: fetch_citation_key("citation_journal_title", html).unwrap(),
            volume: fetch_citation_key("citation_volume", html).unwrap(),
            publisher: fetch_citation_key("citation_publisher", html),
            doi: fetch_citation_key("citation_doi", html),
            isbn: fetch_citation_key("citation_issn", html),
            month: None,
            pages: None,
        };
        bibtex
    }
}

// @article{RN109,
// author = {Domscheit, H. and Hegeman, M. A. and Carvalho, N. and Spieth, P. M.},
// title = {Molecular Dynamics of Lipopolysaccharide-Induced Lung Injury in Rodents},
// journal = {Front Physiol},
// volume = {11},
// pages = {36},
// ISSN = {1664-042X (Print)
// 1664-042X (Linking)},
// DOI = {10.3389/fphys.2020.00036},
// url = {https://www.ncbi.nlm.nih.gov/pubmed/32116752},
// year = {2020},
// type = {Journal Article}
// }

impl Display for Bibtex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "@{}{{{},", self.btype, self.cite_key());
        writeln!(f, "author = {{{}}},", self.authors.replace(";", " and "));
        writeln!(f, "title = {{{}}},", self.title);
        writeln!(f, "journal = {{{}}},", self.journal);
        writeln!(f, "volume = {{{}}},", self.volume);

        if let Some(pages) = &self.pages {
            writeln!(f, "pages = {{{}}},", pages);
        }

        if let Some(publisher) = &self.publisher {
            writeln!(f, "publisher = {{{}}},", publisher);
        }

        if let Some(doi) = &self.doi {
            writeln!(f, "doi = {{{}}},", doi);
        }

        writeln!(f, "year = {{{}}},", self.year);
        writeln!(f, "}}")
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// the title of paper to search
    #[clap(short, long)]
    title: String,

    /// the verbose mode
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

async fn request(url: &str) -> Result<String, reqwest::Error> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

fn fetch_paper_url(html: &str) -> String {
    let fragment = Html::parse_document(html);
    let selector = Selector::parse(r#"a[class="docsum-title"]"#).unwrap();
    let element = fragment.select(&selector).into_iter().next().unwrap();

    format!("{}{}", BASE_URL, element.value().attr("href").unwrap())
}

async fn fetch_paper_info(paper_url: &str) {
    // ref : view-source:https://pubmed.ncbi.nlm.nih.gov/33441414/

    info!("paper url: {}", paper_url);
    let body = request(&paper_url).await;
    let fragment = Html::parse_document(body.unwrap().as_str());
    let bibtex = Bibtex::new(&fragment);
    println!("{}", bibtex);
}

fn fetch_citation_key(key: &str, html: &Html) -> Option<String> {
    let selector = Selector::parse(&*format!(r#"meta[name="{}"]"#, key)).unwrap();
    info!("key: {}", key);
    let element = html.select(&selector).into_iter().next().unwrap();

    if let Some(res) = element.value().attr("content") {
        Some(res.to_string())
    } else {
        None
    }
}

// This is using the `tokio` runtime. You'll need the following dependency:
//
// `tokio = { version = "1", features = ["full"] }`
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cli = Args::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    let title = cli.title;

    let url = format!("{}/?term={}", BASE_URL, title);
    let body = request(&url).await?;

    let paper_url = fetch_paper_url(&body);

    fetch_paper_info(&paper_url).await;

    Ok(())
}

// <a
// class="docsum-title"
// href="/35918585/"
// ref="linksrc=docsum_link&amp;article_id=35918585&amp;ordinalpos=1&amp;page=1"
// data-ga-category="result_click"
// data-ga-action="1"
// data-ga-label="35918585"
// data-full-article-url="from_term=this+is+a+test%5BTitle%5D&amp;from_pos=1"
// data-article-id="35918585">
// This is a <b>test</b>: Oculomotor capture when the experiment keeps score.
// </a>

//
// @article{ahu61,
// author={Arrow, Kenneth J. and Leonid Hurwicz and Hirofumi Uzawa},
// title={Constraint qualifications in maximization problems},
// journal={Naval Research Logistics Quarterly},
// volume={8},
// year=1961,
// pages={175-191}
// }
//

// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
