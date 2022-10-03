use crate::fetch;
use scraper::Html;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Bibtex {
    btype: String,
    title: String,
    author: String,
    journal: String,
    year: String,
    volume: Option<String>,
    number: Option<String>,
    pages: Option<String>,
    publisher: Option<String>,
    doi: Option<String>,
    month: Option<String>,
}

impl Bibtex {
    #[allow(dead_code)]
    fn cite_key(&self) -> String {
        let mut cite_key = String::new();
        let first_author_last_name = &self.author.split_whitespace().next().unwrap();
        cite_key.push_str(first_author_last_name);
        cite_key.push_str(self.year.split_whitespace().next().unwrap());
        cite_key.push_str(self.title.split_whitespace().next().unwrap());
        cite_key.to_lowercase()
    }

    pub fn new(html: &Html) -> Bibtex {
        Bibtex {
            btype: "article".to_string(),
            title: fetch::fetch_title(html).unwrap(),
            author: fetch::fetch_author(html).unwrap(),
            year: fetch::fetch_year(html).unwrap(),
            journal: fetch::fetch_journal(html).unwrap(),
            volume: fetch::fetch_volume(html),
            number: fetch::fetch_number(html),
            pages: fetch::fetch_page(html),
            publisher: fetch::fetch_publisher(html),
            doi: fetch::fetch_doi(html),
            month: None,
        }
    }
}

// @article{uhrig2021accurate,
//   title={Accurate and efficient detection of gene fusions from RNA sequencing data},
//   author={Uhrig, Sebastian and Ellermann, Julia and Walther, Tatjana and Burkhardt, Pauline and Fr{\"o}hlich, Martina and Hutter, Barbara and Toprak, Umut H and Neumann, Olaf and Stenzinger, Albrecht and Scholl, Claudia and others},
//   journal={Genome research},
//   volume={31},
//   number={3},
//   pages={448--460},
//   year={2021},
//   publisher={Cold Spring Harbor Lab}
// }
impl Display for Bibtex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "@{}{{{},", self.btype, self.cite_key())?;
        writeln!(f, "  title = {{{}}},", self.title)?;
        writeln!(f, "  author = {{{}}},", self.author.replace(';', " and "))?;
        writeln!(f, "  journal = {{{}}},", self.journal)?;

        self.volume
            .as_ref()
            .map(|v| writeln!(f, "  volume = {{{}}},", v));
        self.number
            .as_ref()
            .map(|n| writeln!(f, "  number = {{{}}},", n));
        self.pages
            .as_ref()
            .map(|p| writeln!(f, "  pages = {{{}}},", p));
        writeln!(f, "  year = {{{}}},", self.year)?;

        self.publisher
            .as_ref()
            .map(|p| writeln!(f, "  publisher = {{{}}},", p));

        if let Some(doi) = &self.doi {
            writeln!(f, "  doi = {{{}}},", doi)?;
        }

        writeln!(f, "}}")
    }
}
