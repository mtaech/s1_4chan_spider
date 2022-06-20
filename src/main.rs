use scraper::{Html, Selector};
use scraper::html::Select;
use serde::de::Unexpected::Str;

#[derive(Debug)]
struct PageInfo{
    post_list:Vec<String>,
    next_url:String
}
impl PageInfo {
    fn new(post_list: Vec<String>, next_url: String) -> PageInfo {
        PageInfo {
            post_list,
            next_url,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(),reqwest::Error>{
    let select = init_request().await?;
    Ok(())
}

async fn init_request() -> Result<(),reqwest::Error> {
    let url = "http://meipin.im";
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    let doc = Html::parse_document(&html);
    let next_url = get_next_url(&doc).unwrap();
    let post_list = get_post_list(&doc).unwrap();
    let page_info = PageInfo::new(post_list, next_url);
    println!("page info {:?}",page_info);
    Ok(())
}

///获取下一页的跳转链接
fn get_next_url(html:&Html) -> Result<String,reqwest::Error>{
    let next_selector = Selector::parse(".next").unwrap();
    let mut select = html.select(&next_selector);
    let el = select.next().unwrap();
    let url = el.value().attr("href").unwrap();
    Ok(String::from(url))
}

///获取当页所有的贴子链接
fn get_post_list(html:&Html) -> Result<Vec<String>,reqwest::Error>{
    let mut post_url_vec:Vec<String> = Vec::new();
    let post_selector = Selector::parse(r#"a[rel='bookmark']"#).unwrap();
    let select = html.select(&post_selector);
    for el in select {
        let url = el.value().attr("href").unwrap();
        post_url_vec.push(String::from(url));
    }
    Ok(post_url_vec)
}