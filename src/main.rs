// #![deny(warnings)]
use bibtex::Bibtex;
use clap::Parser;
use log::info;
use scraper::{Html, Selector};
use std::io::BufRead;
use std::path::PathBuf;

mod bibtex;
mod fetch;

const BASE_URL: &str = "https://pubmed.ncbi.nlm.nih.gov";

#[allow(unused)]
const META_KEYS: [&str; 10] = [
    "description",
    "citation_title",
    "citation_authors",
    "citation_journal_title",
    "citation_volume",
    "citation_issue",
    "citation_date",
    "citation_publisher",
    "citation_doi",
    "citation_pmid",
];

async fn request(url: &str) -> Result<String, reqwest::Error> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

fn fetch_paper_url(html: &str) -> String {
    let fragment = Html::parse_document(html);

    let test_selector = Selector::parse(r#"meta[name="ncbi_uid"]"#).unwrap();
    let test_element = fragment.select(&test_selector).into_iter().next();

    return if let Some(_test_element) = test_element {
        let uid = _test_element.value().attr("content").unwrap();
        format!("{}/{}/", BASE_URL, uid)
    } else {
        let selector = Selector::parse(r#"a[class="docsum-title"]"#).unwrap();
        let element = fragment.select(&selector).into_iter().next().unwrap();
        format!("{}{}", BASE_URL, element.value().attr("href").unwrap())
    };
}

async fn fetch_paper_info(paper_url: &str) {
    // ref : view-source:https://pubmed.ncbi.nlm.nih.gov/33441414/

    info!("paper url: {}", paper_url);
    let body = request(paper_url).await;
    let fragment = Html::parse_document(body.unwrap().as_str());
    fetch::fetch_page(&fragment);
    let bibtex = Bibtex::new(&fragment);
    println!("{}", bibtex);
}

pub async fn fetch_bibtex(title: String) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/?term={}",
        BASE_URL,
        title.split_whitespace().collect::<Vec<_>>().join("+")
    );
    info!("search url: {}", url);
    let body = request(&url).await?;
    let paper_url = fetch_paper_url(&body);
    fetch_paper_info(&paper_url).await;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// the title of paper to search
    #[clap(short, long)]
    title: Option<String>,

    // the file of papers to search
    #[clap(short, long)]
    file: Option<PathBuf>,

    /// the verbose mode
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
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

    if let Some(title) = cli.title {
        fetch_bibtex(title).await?;
    }

    if let Some(file) = cli.file {
        let file = std::fs::File::open(file).unwrap();
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let title = line.unwrap().trim().to_string();

            fetch_bibtex(title).await?;
        }
    }

    Ok(())
}

// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
