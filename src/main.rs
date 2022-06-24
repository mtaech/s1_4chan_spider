extern crate core;

use home::home_dir;
use log::info;
use scraper::{Html, Node, Selector};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug)]
struct IndexInfo {
    post_list: Vec<PostInfo>,
    next_url: Option<String>,
}
impl IndexInfo {
    fn new(post_list: Vec<PostInfo>, next_url: Option<String>) -> IndexInfo {
        IndexInfo {
            post_list,
            next_url,
        }
    }
}
#[derive(Debug, Clone)]
struct PostInfo {
    title: String,
    date: String,
    url: String,
}

impl PostInfo {
    fn new(title: String, date: String, url: String) -> PostInfo {
        PostInfo { title, date, url }
    }
}

fn main() {
    setup_logger().expect("set logger error");
    let url = "http://meipin.im";
    let info = get_index_info(url).expect("get index info error");
    start_download(info);
}

fn start_download(info: IndexInfo) {
    get_page_content_by_list(info.post_list);
    match info.next_url {
        None => {
            info!("download end");
            panic!("end");
        }
        Some(url) => {
            let info = get_index_info(&url).expect("get index info error");
            info!("start download {:?}", &info.next_url);
            start_download(info)
        }
    }
}

fn get_index_info(url: &str) -> Result<IndexInfo, reqwest::Error> {
    let response = reqwest::blocking::get(url).expect("get page info error");
    let html = response.text().expect("get html error");
    let doc = Html::parse_document(&html);
    let next_url = get_next_url(&doc).unwrap();
    let post_list = get_post_info(&doc).unwrap();
    let page_info = IndexInfo::new(post_list, Some(next_url));
    Ok(page_info)
}

///获取下一页的跳转链接
fn get_next_url(html: &Html) -> Result<String, reqwest::Error> {
    let next_selector = Selector::parse(".next").unwrap();
    let mut select = html.select(&next_selector);
    let el = select.next().unwrap();
    let url = el.value().attr("href").unwrap();
    Ok(String::from(url))
}

///获取当页所有的贴子链接
fn get_post_info(html: &Html) -> Result<Vec<PostInfo>, reqwest::Error> {
    let mut post_vec: Vec<PostInfo> = Vec::new();
    let post_selector = Selector::parse("article").unwrap();
    let select = html.select(&post_selector);
    for el in select {
        //获取标题
        let title_selector = Selector::parse("header a[rel='bookmark']").unwrap();
        let title_el = el.select(&title_selector).next().unwrap();
        let title = title_el.inner_html();
        let url = title_el.value().attr("href").unwrap();
        //获取发布时间
        let date_selector = Selector::parse("time.entry-date").unwrap();
        let mut date = el.select(&date_selector).next().unwrap().inner_html();
        date = date.replace('/', "-");
        post_vec.push(PostInfo::new(title, date, String::from(url)));
    }
    Ok(post_vec)
}
fn get_page_content_by_list(post_list: Vec<PostInfo>) {
    for post in post_list {
        get_post_content(post).expect("get post content error");
    }
}

fn get_post_content(info: PostInfo) -> Result<(), reqwest::Error> {
    info!("start download post:{:?}", info);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let request = client.get(&info.url).build().unwrap();
    let response = client.execute(request).unwrap();
    let doc = response.text().expect("get context error");
    let html = Html::parse_document(&doc);
    let article_selector = Selector::parse("article").unwrap();
    let article_el = html.select(&article_selector).next().unwrap();
    let nodes = article_el.tree().nodes();
    let post_dir = get_post_dir(&info.date, &info.title);
    let mut title_flag = false;
    let mut meipintu_index = 0;
    let mut title_str = "".to_string();
    let mut meipin_flag = false;
    let mut meipin_text = String::from("");
    let mut meipin_add_flag = false;
    for node in nodes {
        match node.value() {
            Node::Document => {}
            Node::Fragment => {}
            Node::Doctype(_doc_type) => {}
            Node::Comment(_comment) => {}
            Node::Text(text) => {
                if title_flag {
                    title_str = text.text.to_string();
                    title_flag = false;
                } else {
                    let mut txt = text.text.to_string();
                    txt = txt.trim().to_string();
                    if !txt.eq("\n") && !txt.eq("\n\u{3000}") && !txt.is_empty() {}
                    if !meipin_flag && txt.starts_with("没品") {
                        meipin_flag = true;
                    }
                    if meipin_flag && meipin_add_flag && !txt.is_empty() {
                        meipin_text += txt.as_ref();
                        meipin_text += "\n";
                    }
                }
            }
            Node::Element(ele) => match ele.name() {
                "img" => {
                    let src = ele.attr("src").expect("get img sec error");
                    if title_str.starts_with("没品选段") {
                        let suffix = meipintu_index.to_string();
                        title_str = String::from("没品选段-") + suffix.as_ref();
                        meipintu_index += 1;
                    }
                    println!("title : {:?} src:{:?}", title_str, src);
                    let info_vec: Vec<&str> = src.split('.').collect();
                    let suffix = info_vec.last().expect("get suffix error");
                    if !suffix.is_empty() {
                        title_str = title_str + "." + suffix;
                    }
                    download_img(src, &title_str, &post_dir);
                    meipin_add_flag = false;
                }
                "h4" => {
                    title_flag = true;
                    meipin_add_flag = false;
                }
                "p" => {
                    meipin_add_flag = true;
                }
                "a" => {
                    let href = ele.attr("href").expect("get href error");
                    if href.contains("sickipedia.net") {
                        meipin_add_flag = true;
                    } else {
                        meipin_add_flag = false;
                    }
                }
                "br" => {
                    meipin_add_flag = true;
                }
                _ => {
                    meipin_add_flag = false;
                }
            },
            Node::ProcessingInstruction(_ins) => {}
        }
    }
    if !meipin_text.is_empty() {
        save_to_file(meipin_text, &post_dir);
    }
    info!("end download post:{:?}", info);
    Ok(())
}

fn get_post_dir(data: &str, title: &str) -> PathBuf {
    let home_dir = home_dir().expect("get home path error");
    let buf = home_dir
        .join("Documents")
        .join("4chan")
        .join(data.to_string() + "-" + title);
    if !buf.exists() {
        fs::create_dir_all(&buf).expect("create dir error");
    }
    buf
}
fn download_img(url: &str, title: &String, dir_path: &Path) {
    let img_path = dir_path.join(title);
    if !img_path.exists() {
        info!("start download image: {:?}", url);
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        let request = client.get(url).build().unwrap();
        let img_resp = client.execute(request).expect("get image error");
        let mut file = File::create(&img_path).expect("create img path filed");
        let stream = img_resp.bytes().expect("get img bytes error");
        let _ = file.write_all(stream.as_ref()).expect("save image error");
        info!("end download image: {:?}", url);
    }
}

fn save_to_file(meipin_text: String, dir_path: &Path) {
    let text_path = dir_path.join("没品选段.txt");
    info!("start write text : {:?}", text_path);
    let mut file = File::create(&text_path).expect("create img path filed");
    let _ = file.write_all(meipin_text.as_bytes());
    info!("end write text : {:?}", text_path);
}

fn setup_logger() -> Result<(), fern::InitError> {
    let log_dir = home_dir()
        .expect("get home path error")
        .join("Documents")
        .join("4chan");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("log dir create error");
    }
    let log_path = log_dir.join("4chan.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path)?)
        .apply()?;
    Ok(())
}
