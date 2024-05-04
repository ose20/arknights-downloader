use std::{fs, path::Path};

use anyhow::{Ok, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use url::Url;

// キャラクター一覧のurl、動作確認用に一旦ハードコーティングする
const SRC_URL: &str = "https://arknights.wikiru.jp/?%E3%82%AD%E3%83%A3%E3%83%A9%E3%82%AF%E3%82%BF%E3%83%BC%E4%B8%80%E8%A6%A7";
// ダウンロード先のディレクトリ名
const TAR_DIR: &str = "results/operators";
const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";

pub fn run() -> Result<()> {
    fs::create_dir_all(TAR_DIR)?;

    let client = Client::new();
    let response = client
        .get(SRC_URL)
        .header(reqwest::header::USER_AGENT, AGENT)
        .send()?;

    let html = response.text()?;
    let document = Html::parse_document(&html);

    let img_elts = select_operators_elements(&document);

    for img_elt in img_elts {
        let img_src = img_elt.value().attr("data-src").unwrap();
        let img_url = Url::parse(SRC_URL)?.join(img_src)?;
        let file_name = sanitize_filename(img_elt.value().attr("title").unwrap());
        let ext = Path::new(img_src).extension().unwrap().to_str().unwrap();

        let mut response = client.get(img_url).send()?;
        let mut file = fs::File::create(format!("{}/{}.{}", TAR_DIR, file_name, ext))?;
        response.copy_to(&mut file)?;
    }

    Ok(())
}

fn select_operators_elements(document: &Html) -> Vec<scraper::element_ref::ElementRef> {
    let start_id = 1;
    let end_id = 12;
    let mut selectors = Vec::new();

    for i in start_id..=end_id {
        selectors.push(format!("#sortabletable{} img.lazyload[data-src]", i));
    }

    let selector = Selector::parse(&selectors.join(", ")).unwrap();
    document.select(&selector).collect()
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(
        |c: char| {
            c == '/'
                || c == '\\'
                || c == ':'
                || c == '*'
                || c == '?'
                || c == '"'
                || c == '<'
                || c == '>'
                || c == '|'
        },
        "-",
    )
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::SRC_URL;

    #[test]
    fn test_url() {
        let url = Url::parse(SRC_URL).unwrap();
        assert_eq!(url.to_string(), SRC_URL.to_string());
        // join をすると、url の最後の / 以後は remove される
        assert_eq!(
            url.join("test").unwrap().to_string(),
            "https://arknights.wikiru.jp/test".to_string()
        )
    }
}
