use std::fs::File;

use select::document::Document;
use select::node::Node;
use select::predicate::*;

use std::io::Error;
use std::io::{self, Write};

use reqwest::get;


#[derive(Debug)]
struct Icon{
    x: u8,
    y: u8,
    href: String
}

fn attr_parser(doc: &Document, attr: &str, val: &str) -> Vec<Icon>{
    let mut links: Vec<Icon> = Vec::new();
    
    for link in doc.find(Name("link").and(Attr(attr, val))).collect::<Vec<Node>>(){
        println!("{:?}", link);
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
    links.extend(attr_parser(doc, "rel", "apple-touch-icon image_src"));
    links.sort_by_key(|a| a.x);

    let out = links.last();
    let out = match out{
        Some(links) => Some(links.href.clone()),
        None => None
    };
    out 
    // println!("{:?}", links);
    // if links.len() == 0{
    //     None
    // } else
    // {}
    // links.last().unwrap().href.clone()
}

pub fn get_image(url: &str) -> Result<String, Error>{
    let mut resp = get(url).expect("Fail2");
    let body = resp.text().expect("Body fail");
    
    let document = Document::from(&body[..]);
    
    let url = get_image_paths(&document);
    return Ok(url.unwrap())
}


pub fn download_media(url: &str, fs_path: &str){
    let mut resp = get(url).expect("Fail3");
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf).expect("Bad body");
    println!("{}", fs_path);
    let mut f = File::create(fs_path).unwrap();
    f.write_all(buf.as_slice());
}
