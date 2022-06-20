use std::borrow::Borrow;
use scraper::{ElementRef, Html, Node, Selector};
use scraper::node::Text;

#[derive(Debug)]
struct IndexInfo {
    post_list:Vec<PostInfo>,
    next_url:Option<String>
}
impl IndexInfo {
    fn new(post_list: Vec<PostInfo>, next_url: Option<String>) -> IndexInfo {
        IndexInfo {
            post_list,
            next_url
        }
    }
}
#[derive(Debug, Clone)]
struct PostInfo {
    title: String,
    date: String,
    url: String
}

impl PostInfo {
    fn new(title:String,date:String,url:String) -> PostInfo {
        PostInfo {
            title,
            date,
            url
        }
    }
}

#[tokio::main]
async fn main() -> Result<(),reqwest::Error>{
    let url = "http://meipin.im";
    let info = get_page_info(url).await?;
    get_page_content_by_list(&info.post_list).await;
    Ok(())
}

async fn get_page_info(url:&str) -> Result<IndexInfo,reqwest::Error> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    let doc = Html::parse_document(&html);
    let next_url = get_next_url(&doc).unwrap();
    let post_list = get_post_info(&doc).unwrap();
    let page_info = IndexInfo::new(post_list, Some(next_url));
    println!("page info {:?}",page_info);
    Ok(page_info)
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
fn get_post_info(html:&Html) -> Result<Vec<PostInfo>,reqwest::Error>{
    let mut post_vec:Vec<PostInfo> = Vec::new();
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
async fn get_page_content_by_list(post_list:&Vec<PostInfo>){
    /*for post in post_list {
        get_post_content(post.url).await.unwrap();
    }*/
    // let url = &*post_list.get(0).unwrap().url;
    get_post_content("http://meipin.im/p/101556").await.unwrap();
}

async fn get_post_content(url:&str) -> Result<(),reqwest::Error> {
    let response = reqwest::get(url).await?;
    let doc = response.text().await?;
    let html = Html::parse_document(&doc);
    let content_selector = Selector::parse(".entry-content").unwrap();
    let content_el = html.select(&content_selector).next().unwrap();
    for node in content_el.children() {
        match node.value().as_text() {
            None => {}
            Some(text) => {
                println!("text:{:?}",text)
            }
        };
    }
    Ok(())
}