use std::fs::File;

use select::document::Document;
use select::node::Node;
use select::predicate::*;

use std::io;
use std::io::Write;

use reqwest::Client;
use reqwest::get;
use reqwest::header;

use url::Url;

#[derive(Debug)]
struct Icon{
    x: u8,
    y: u8,
    href: String
}

fn attr_parser(doc: &Document, attr: &str, val: &str) -> Vec<Icon>{
    let mut links: Vec<Icon> = Vec::new();

    
    for link in doc.find(Name("link").and(Attr(attr, val))).collect::<Vec<Node>>(){
        
        if !link.attr("href").unwrap().ends_with("png") {
            continue
        }
        
        let sizes = link.attr("sizes");
        let sizes = match sizes{
            Some(s) => s,
            // if it hasn't got a size let's treat it as the smallest
            None => "1x1"
        };
        
        let x_y = sizes.split("x").collect::<Vec<&str>>();
        let x = x_y[0].parse::<u8>().unwrap();
        let y = x_y[0].parse::<u8>().unwrap();

        links.push(Icon{x, y, href: String::from(link.attr("href").unwrap())});
    };
    links
}

fn get_image_paths(doc: &Document) -> Option<String>{
    let mut links: Vec<Icon> = Vec::new();
    
    links.extend(attr_parser(doc, "rel", "icon"));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon-precomposed"));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon"));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon image_src"));
    links.sort_by_key(|a| a.x);

    if links.len() == 0{
        //println!("link len 0");
        //  let nodes = doc.find(Name("meta")).collect::<Vec<Node>>();
        //  println!("{:?}", nodes);
        for link in doc.find(Name("meta").and(Attr("property", "og:image"))).collect::<Vec<Node>>(){
        //or link in nodes {
            links.push(Icon{x: 1, y: 1, href: String::from(link.attr("content").unwrap())});
            
        };
    };

    
    let out = links.last();
    let out = match out{
        Some(links) => Some(links.href.clone()),
        None => None
    };
    out 
}

fn url_from_paths(root: &str, path: &str) -> String{
    let mut image_url = String::from(path);
    if &image_url[..2] == "//"{
        image_url.insert_str(0, "http:");
        return image_url
    }
    
    if image_url.len() >= 4 && &image_url[..4] == "http"{
       return image_url
    }              
    
    let root_url = Url::parse(&root).unwrap();
    let parsed = root_url.join(&path).unwrap();
    String::from(parsed.as_str())
}

pub fn download_image(url: &str, fs_path: &str) -> Result<(), io::Error>{
    let mut headers = header::Headers::new();
    headers.set(header::UserAgent::new("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/44.0.2403.89 Safari/537.36".to_string()));

    // get a client builder
    let client = Client::builder()
        .default_headers(headers)
        .build().expect("Bob");
    let mut resp = client.get(url)
        .header(header::UserAgent::new("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/61.0.3163.79 Safari/537.36".to_string()))
        .send().expect("Fail2");
    
    //let mut resp = get(url).expect("Fail2");
    let body = resp.text().expect("Body fail");

    let document = Document::from(&body[..]);
    
    let image_url = get_image_paths(&document).unwrap();
    let download_url = url_from_paths(url, &image_url);
    download_media(&download_url, fs_path);
    
    return Ok(())
}


pub fn download_media(url: &str, fs_path: &str){
    println!("{}", &url);
    let mut resp = get(url).expect("Fail3");
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf).expect("Bad body");
    let mut f = File::create(fs_path).unwrap();
    f.write_all(buf.as_slice()).unwrap();
}


#[test]
fn url_from_paths_test(){
    assert_eq!("https://www.example.com/123",
    url_from_paths("https://www.example.com", "123"));
    assert_eq!("http://www.example2.com/123",
    url_from_paths("https://www.example.com", "http://www.example2.com/123"));
    assert_eq!("http://www.example2.com/123",
    url_from_paths("https://www.example.com", "//www.example2.com/123"));
}
