use std::fs::File;

use select::document::Document;
use select::node::Node;
use select::predicate::*;

use std::io::Write;

use reqwest::Client;
use reqwest::get;
use reqwest::header;

use serde_json;
use serde_json::Value;
use reqwest;

use url::Url;

#[derive(Debug, Clone, PartialEq)]
struct Icon{
    x: u16,
    y: u16,
    href: String,
    poor: bool
}

const DESKTOP_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/61.0.3163.79 Safari/537.36";

const MOBILE_UA: &str = "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/64.0.3282.119 Mobile Safari/537.36";

fn split_x_y(sizes: &str) -> (u16, u16){
    let x_y = sizes.split('x').collect::<Vec<&str>>();
    if x_y.len() != 2 {
        return (1, 1)
    }
    let x = x_y[0].parse::<u16>().unwrap();
    let y = x_y[1].parse::<u16>().unwrap();
    (x, y)
}

#[test]
fn split_x_y_test(){
    assert_eq!((1, 2), split_x_y("1x2"));
    assert_eq!((1, 1), split_x_y("any"));
}


fn attr_parser(doc: &Document, _attr: &str, _val: &str, url: &str) -> Vec<Icon>{
    let mut links: Vec<Icon> = Vec::new();

    for link in doc.find(Name("link")).collect::<Vec<Node>>(){
        let href = match link.attr("href"){
            Some(s) => s,
            None => continue
        };
        
        if !href.ends_with("png") {
            continue
        }
        
        let sizes = match link.attr("sizes"){
            Some(s) => s,
            // if it hasn't got a size let's treat it as the smallest
            None => "1x1"
        };

        let (x, y) = split_x_y(sizes);
        let path = link.attr("href").unwrap();
        if path.is_empty() {
            continue
        }
        links.push(Icon{x, y, href: url_from_paths(url, path), poor: false});
    };
    links
}

#[test]
fn attr_parser_test(){
    let doc1 = Document::from("<html><head><link rel=\"icon\" sizes=\"192x192\" href=\"/1.png\"/></head></html>");
    assert_eq!(vec![Icon{x: 192, y: 192, href: "http://example.com/1.png".to_string(), poor: false}], attr_parser(&doc1, "", "", "http://example.com"));

    let doc2 = Document::from("<html><head><link rel=\"icon\" sizes=\"192x192\" href=\"/1.svg\"/></head></html>");
    let a: Vec<Icon> = Vec::new();
    
    assert_eq!(a, attr_parser(&doc2, "", "", "http://example.com"));

    
        
}

fn icons_from_manifest(url: &str, data: &str) -> Option<Vec<Icon>>{
    let mut icons: Vec<Icon> = Vec::new();
    match serde_json::from_str(data) {
        Ok(a) => {
            let b: Value = a;
            if let Some(x) = b["icons"].as_array() {
                for link in x.iter() {
                    let sizes = link["sizes"].as_str().unwrap();
                    let (x, y) = split_x_y(sizes);

                    let mut src = String::from(link["src"].as_str().unwrap());
                    if !src.starts_with("http") {
                        let mut divider = "";
                        if !url.ends_with("/") && !&src.starts_with("/") {
                            divider = "/";
                        }
                            
                        src = url.to_owned() + &divider +  &src;
                            
                    }
                    
                    icons.push(Icon{x, y,
                                    href: src,
                                    poor:false});

                };
                if !icons.is_empty() {
                    return Some(icons)
                }
                return None
            }
            None
        },
        _ => None
    }
}
    
#[test]
fn icons_from_manifest_test(){
    assert_eq!(vec![
        Icon{x: 114, y: 114, href: String::from("https://assets-cdn.github.com/apple-touch-icon-114x114.png"), poor: false},
        Icon{x: 120, y: 120, href: String::from("https://assets-cdn.github.com/apple-touch-icon-120x120.png"), poor: false}],
               icons_from_manifest("http://www.example.com", "{\"name\":\"GitHub\",\"icons\":[{\"sizes\":\"114x114\",\"src\":\"https://assets-cdn.github.com/apple-touch-icon-114x114.png\"},{\"sizes\":\"120x120\",\"src\":\"https://assets-cdn.github.com/apple-touch-icon-120x120.png\"}]}").unwrap())}
 

fn get_image_paths(doc: &Document, url: &str) -> Option<Vec<Icon>>{
    let mut links: Vec<Icon> = Vec::new();
    
    links.extend(attr_parser(doc, "rel", "icon", url));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon-precomposed", url));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon", url));
    links.extend(attr_parser(doc, "rel", "apple-touch-icon image_src", url));
    links.sort_by_key(|a| a.x);
    
    if links.is_empty() {
        for link in doc.find(Name("meta").and(Attr("property", "og:image"))).collect::<Vec<Node>>(){
            let path = link.attr("content").unwrap();
            if path.is_empty() {
                continue
            }
            links.push(Icon{x: 1,
                            y: 1,
                            href: url_from_paths(url, link.attr("content").unwrap()),
                            poor: true});
        };
    };
    
    if links.is_empty() {
        None
    } else {
        Some(links)
    }
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
    
    let root_url = Url::parse(root).unwrap();
    let parsed = root_url.join(path).unwrap();
    String::from(parsed.as_str())
}

fn document_for_ua(url: &str, ua: &str) -> Result<(String, Document), reqwest::Error>{
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(&ua.to_string()).unwrap());

    // get a client builder
    let client = Client::builder()
        .default_headers(headers)
        .build().expect("Bob");
    let mut resp = try!(client.get(url)
        .send());

    let urlout = String::from(resp.url().as_str());
    let body = try!(resp.text());

    Ok((urlout, Document::from(&body[..])))
}

// TODO: Change to Option
fn get_manifest_json(url: &str, ua: &str) -> Result<String, reqwest::Error>{
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(&ua.to_string()).unwrap());

    let mut root_url = Url::parse(url).unwrap();
    root_url.set_path(""); 
    let parsed = root_url.join("manifest.json").unwrap();
    // get a client builder
    let client = Client::builder()
        .default_headers(headers)
        .build().expect("Bob");
    let mut resp = try!(client.get(parsed)
        .send());
    
    let body = try!(resp.text());
    Ok(body)
}


fn get_mobile_icons(url: &str, icons: Option<&[Icon]>) -> Option<Icon>{
    let (final_url, md) = document_for_ua(url, MOBILE_UA).unwrap();
    match get_image_paths(&md, &final_url) {
        Some(mut mis) => {
            mis.sort_by_key(|a| a.x);
            Some(mis.last().unwrap().clone())
        },
        None => {
            match icons {
                Some(i) => Some(i.last().unwrap().clone()),
                None => None
            }
        }
    }
}
   

fn good_quality_or_mobile(icons: &[Icon], url: &str) -> Option<Icon>{
    println!("{:?}", icons);
    let out = icons.last().unwrap().clone();
    if out.x > 128{
        Some(out.clone())
    } else {
        get_mobile_icons(url, Some(icons))
    }
}

fn get_page_header_icons(url: &str) -> Result<Option<Icon>, reqwest::Error>{
    let (final_url, document) = try!(document_for_ua(url, DESKTOP_UA));
    
    match get_image_paths(&document, &final_url) {
        Some(icons) => Ok(good_quality_or_mobile(&icons, url)),
        None => Ok(get_mobile_icons(url, None)) 
    }
}


fn get_icon_objects(url: &str) -> Result<Option<Icon>, reqwest::Error>{
    let manifest_test = get_manifest_json(url, DESKTOP_UA);

    let icon = match manifest_test {
        Ok(data) => {
            match icons_from_manifest(&url, &data) {
                Some(mut icon) => {
                    icon.sort_by_key(|a| a.x);
                    Some(icon.last().unwrap().clone())
                },
                _ => None
            }
        },
        _ => None
    };

    match icon {
        Some(_) => Ok(icon),
        None => get_page_header_icons(url)
    }
}


pub fn download_image(url: &str, fs_path: &str) -> Result<(), reqwest::Error>{
    let icon_url = match get_icon_objects(url){
        Ok(links) => {
            match links {
                Some(i) => Some(i.href.clone()),
                _ => None}},
            
        _ => None};

    match icon_url {
        Some(image_url) => {
            download_media(&image_url, fs_path)
        },
        _  => Ok(println!("No image for {}", url))
    }
}


pub fn download_media(url: &str, fs_path: &str) -> Result<(), reqwest::Error>{
    let mut resp = get(url)?;
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf).expect("Bad body");
    let mut f = File::create(fs_path).unwrap();
    f.write_all(buf.as_slice()).unwrap();
    Ok(())
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


