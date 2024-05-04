use std::{fs, path::Path};

use anyhow::{Ok, Result};
use reqwest::blocking::Client;
use scraper::{element_ref, Html, Selector};
use url::Url;

use clap::Parser;

// ----------------------------------------------------------------------------------------------------
const BASE_URL: &str = "https://arknights.wikiru.jp/";
const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";

// ----------------------------------------------------------------------------------------------------
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// オペレータをダウンロードします
    #[arg(long)]
    operator: bool,
    /// 合成玉や龍門幣などの貴重品をダウンロードします
    #[arg(long)]
    valuable: bool,
    /// モジュールデータや中級異鉄などの育成素材をダウンロードします
    #[arg(long)]
    material: bool,
    /// 作戦記録とアーツ学をダウンロードします
    #[arg(long)]
    exp_arts: bool,
    /// SoCをダウンロードします
    #[arg(long)]
    soc: bool,
}

// ----------------------------------------------------------------------------------------------------
impl Args {
    fn get_targets(&self) -> Vec<TargetConfig> {
        let mut targets = vec![];

        if self.operator {
            targets.push(Target::Operator.to_config());
        }
        if self.valuable {
            targets.push(Target::Valuable.to_config());
        }
        if self.material {
            targets.push(Target::Material.to_config());
        }
        if self.exp_arts {
            targets.push(Target::ExpArts.to_config());
        }
        if self.soc {
            targets.push(Target::Soc.to_config());
        }

        targets
    }
}

// ----------------------------------------------------------------------------------------------------
/// download する対象を表現する型
struct TargetConfig {
    target: Target,
    src_url: String,
    tar_dir: String,
}

// ----------------------------------------------------------------------------------------------------
impl TargetConfig {
    fn download(&self) -> Result<()> {
        match self.target {
            Target::Operator => {
                fs::create_dir_all(&self.tar_dir)?;
                let client = Client::new();
                let response = client
                    .get(&self.src_url)
                    .header(reqwest::header::USER_AGENT, AGENT)
                    .send()?;

                let document = Html::parse_document(&response.text()?);
                let img_elts = self.select_elemtens(&document);
                for img_elt in img_elts {
                    let img_src = img_elt.value().attr("data-src").unwrap();
                    let img_url = Url::parse(BASE_URL)?.join(img_src)?;
                    let filename = sanitize_filename(img_elt.value().attr("title").unwrap());
                    let ext = Path::new(img_src).extension().unwrap().to_str().unwrap();

                    let mut response = client.get(img_url).send()?;
                    let mut file =
                        fs::File::create(format!("{}/{}.{}", self.tar_dir, filename, ext))?;
                    response.copy_to(&mut file)?;
                }
            }
            Target::Valuable => {}
            Target::Material => {}
            Target::ExpArts => {}
            Target::Soc => {}
        }

        Ok(())
    }

    fn select_elemtens<'a>(&'a self, document: &'a Html) -> Vec<element_ref::ElementRef> {
        match self.target {
            Target::Operator => {
                let start_id = 1;
                let end_id = 12;
                let css_selector = (start_id..=end_id)
                    .map(|i| format!("#sortabletable{} img.lazyload[data-src]", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                let selector = Selector::parse(&css_selector).unwrap();
                document.select(&selector).collect()
            }
            Target::Valuable => {
                unimplemented!()
            }
            Target::Material => {
                unimplemented!()
            }
            Target::ExpArts => {
                unimplemented!()
            }
            Target::Soc => {
                unimplemented!()
            }
        }
    }
}

// ----------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
enum Target {
    Operator,
    Valuable,
    Material,
    ExpArts,
    Soc,
}

// ----------------------------------------------------------------------------------------------------
impl Target {
    fn to_config(self) -> TargetConfig {
        match self {
            Self::Operator => {
                TargetConfig {
                    target: self,
                    src_url: "https://arknights.wikiru.jp/?%E3%82%AD%E3%83%A3%E3%83%A9%E3%82%AF%E3%82%BF%E3%83%BC%E4%B8%80%E8%A6%A7".to_string(),
                    tar_dir: "results/operators".to_string()
                }
            },
            Self::Valuable => {
                TargetConfig {
                    target: self,
                    src_url: "https://arknights.wikiru.jp/?%E3%83%86%E3%83%BC%E3%83%96%E3%83%AB/%E8%B2%B4%E9%87%8D%E5%93%81".to_string(),
                    tar_dir: "results/valuables".to_string()
                }
            },
            Self::Material => {
                TargetConfig {
                    target: self,
                    src_url: "https://arknights.wikiru.jp/?%E3%83%86%E3%83%BC%E3%83%96%E3%83%AB/%E7%B4%A0%E6%9D%90".to_string(),
                    tar_dir: "results/materials".to_string()
                }
            },
            Self::ExpArts => {
                TargetConfig {
                    target: self,
                    src_url: "https://arknights.wikiru.jp/?%E3%83%86%E3%83%BC%E3%83%96%E3%83%AB/%E4%BD%9C%E6%88%A6%E8%A8%98%E9%8C%B2%E3%83%BB%E3%82%A2%E3%83%BC%E3%83%84%E5%AD%A6".to_string(),
                    tar_dir: "results/exp-arts".to_string()
                }
            },
            Self::Soc => {
                TargetConfig {
                    target: self,
                    src_url: "https://arknights.wikiru.jp/?%E3%83%86%E3%83%BC%E3%83%96%E3%83%AB/SoC".to_string(),
                    tar_dir: "results/soc".to_string(),
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------------------------------
pub fn run() -> Result<()> {
    let targets = Args::parse().get_targets();
    for target in targets.iter() {
        target.download()?;
    }

    Ok(())
}

// ----------------------------------------------------------------------------------------------------
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
