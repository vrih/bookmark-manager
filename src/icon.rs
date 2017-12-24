use std::fs::File;

use select::document::Document;
use select::node::Node;
use select::predicate::*;

use std::io::Error;
use std::io::Write;

use reqwest::Client;
use reqwest::get;
use reqwest::header;

#[derive(Debug)]
struct Icon{
    x: u8,
    y: u8,
    href: String
}

fn attr_parser(doc: &Document, attr: &str, val: &str) -> Vec<Icon>{
    let mut links: Vec<Icon> = Vec::new();
    
    for link in doc.find(Name("link").and(Attr(attr, val))).collect::<Vec<Node>>(){
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

pub fn get_image(url: &str) -> Result<String, Error>{
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
    
    let url = get_image_paths(&document);
    return Ok(url.unwrap())
}


pub fn download_media(url: &str, fs_path: &str){
    let mut resp = get(url).expect("Fail3");
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf).expect("Bad body");
    let mut f = File::create(fs_path).unwrap();
    f.write_all(buf.as_slice()).unwrap();
}
