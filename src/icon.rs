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

fn split_x_y(sizes: &str) -> (u16, u16) {
    let x_y = sizes.split('x').collect::<Vec<&str>>();
    if x_y.len() != 2 {
        return (1, 1)
    }
    let x_y_parsed = x_y.iter().map(|&x| x.parse::<u16>().unwrap()).collect::<Vec<u16>>();
    (x_y_parsed[0], x_y_parsed[1])
}

fn attr_parser(doc: &Document, _attr: &str, _val: &str, url: &str) -> Vec<Icon>{
    let mut links: Vec<Icon> = Vec::new();

    for link in doc.find(Name("link")).collect::<Vec<Node>>(){
        let href = match link.attr("href"){
            Some(s) => s,
            None => continue
        };
        
        if !href.ends_with("png") && !href.ends_with("svg") {
            continue
        }
        
        let (x, y) = split_x_y(link.attr("sizes").unwrap_or("1x1"));
        let path = link.attr("href").unwrap();
        if path.is_empty() {
            continue
        }
        links.push(Icon{x, y, href: url_from_paths(url, path), poor: false});
    };
    links
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
                    
                    icons.push(Icon{x, y, href: src, poor:false});
                };

                if !icons.is_empty() { return Some(icons) };
                return None
            }
            None
        },
        _ => None
    }
}

fn get_image_paths(doc: &Document, url: &str) -> Option<Vec<Icon>>{
    let mut links: Vec<Icon> = Vec::new();

    for x in [
        "icon",
        "apple-touch-icon-precomposed",
        "apple-touch-icon",
        "apple-touch-icon image_src",
    ].iter() { links.extend(attr_parser(doc, "rel", x, url)) }
    
    if links.is_empty() {
        for link in doc.find(Name("meta").and(Attr("property", "og:image"))).collect::<Vec<Node>>(){
            let path = link.attr("content").unwrap();
            if path.is_empty() {
                continue
            }
            links.push(Icon{x: 1, y: 1, href: url_from_paths(url, path), poor: true});
        };
    };
    
    if links.is_empty() {
        None
    } else {
        links.sort_by_key(|a| a.x);
        Some(links)
    }
}

fn url_from_paths(root: &str, path: &str) -> String{
    let mut image_url = String::from(path);
    if image_url.starts_with("//") {
        image_url.insert_str(0, "http:");
        return image_url
    }
    
    if image_url.starts_with("http") {
       return image_url
   }              
    
    return String::from(Url::parse(root).unwrap().join(path).unwrap().as_str());
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

fn get_best_icon(all_icons: &mut Vec<Icon>, new_icons: &Vec<Icon>) -> Icon {
    all_icons.extend(new_icons.to_owned());
    all_icons.sort_by_key(|a| a.x);
    return all_icons.last().unwrap().to_owned();
}

fn get_page_header_icons(url: &str) -> Result<Option<Icon>, reqwest::Error>{
    let mut all_icons: Vec<Icon> = Vec::new();

    let (final_url, document) = try!(document_for_ua(url, DESKTOP_UA));
    let best_desktop_icon = get_image_paths(&document, &final_url)
        .map(|i| get_best_icon(&mut all_icons, &i));

    match best_desktop_icon {
        Some(icon) => if icon.x > 128 { return Ok(Some(icon)) },
        None => ()
    };

    let (final_urlb, documentb) = try!(document_for_ua(url, MOBILE_UA));
    return Ok(get_image_paths(&documentb, &final_urlb)
              .map(|i| get_best_icon(&mut all_icons, &i)));
}

fn get_icon_objects(url: &str) -> Result<Option<Icon>, reqwest::Error>{
    let manifest_test = get_manifest_json(url, DESKTOP_UA);

    let icon = match manifest_test {
        Ok(data) => {
            icons_from_manifest(&url, &data).map(|i| {
                let mut all_icons: Vec<Icon> = Vec::new();
                get_best_icon(&mut all_icons, &i)
            })
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
        Ok(links) => links.map(|i| i.href.clone()),
        _ => None};

    match icon_url {
        Some(image_url) => {
            download_media(&image_url, fs_path)
        },
        _  => Ok(println!("No image for {}", url))
    }
}

fn replace_extension(source: &str, dest: &str) -> String {
    if source.ends_with("svg") {
        return dest.replace(".png", ".svg");
    }
    return dest.to_owned();
}

pub fn download_media(url: &str, fs_path: &str) -> Result<(), reqwest::Error>{
    let mut resp = get(url)?;
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf).expect("Bad body");
    let amended_path = replace_extension(url, fs_path);
    let mut f = File::create(amended_path).unwrap();
    f.write_all(buf.as_slice()).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_x_y_test(){
        assert_eq!((1, 2), split_x_y("1x2"));
        assert_eq!((1, 1), split_x_y("any"));
    }

    #[test]
    fn icons_from_manifest_test(){
        assert_eq!(vec![
            Icon{x: 114, y: 114, href: String::from("https://assets-cdn.github.com/apple-touch-icon-114x114.png"), poor: false},
            Icon{x: 120, y: 120, href: String::from("https://assets-cdn.github.com/apple-touch-icon-120x120.png"), poor: false}],
                   icons_from_manifest("http://www.example.com", "{\"name\":\"GitHub\",\"icons\":[{\"sizes\":\"114x114\",\"src\":\"https://assets-cdn.github.com/apple-touch-icon-114x114.png\"},{\"sizes\":\"120x120\",\"src\":\"https://assets-cdn.github.com/apple-touch-icon-120x120.png\"}]}").unwrap())}

    #[test]
    fn attr_parser_test(){
        let doc1 = Document::from("<html><head><link rel=\"icon\" sizes=\"192x192\" href=\"/1.png\"/></head></html>");
        assert_eq!(vec![Icon{x: 192, y: 192, href: "http://example.com/1.png".to_string(), poor: false}], attr_parser(&doc1, "", "", "http://example.com"));
        
        let doc2 = Document::from("<html><head><link rel=\"icon\" sizes=\"192x192\" href=\"/1.bad\"/></head></html>");
        let a: Vec<Icon> = Vec::new();
        
        assert_eq!(a, attr_parser(&doc2, "", "", "http://example.com"));
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

    #[test]
    fn replace_extension_test() {
        assert_eq!("123.png", replace_extension("abc.png", "123.png"));
        assert_eq!("123.svg", replace_extension("abc.svg", "123.png"));
        assert_eq!("123.png", replace_extension("abc.ico", "123.png"));
    }

    #[test]
    fn get_best_icon_test() {
        let mut all_icons: Vec<Icon> = Vec::new();
        let icons = vec![
            Icon{x: 2, y: 2, href:"a".to_string(), poor: false},
            Icon{x: 1, y: 1, href:"a".to_string(), poor: false}
        ];
        assert_eq!(Icon{x: 2, y: 2, href: "a".to_string(), poor: false},
                   get_best_icon(&mut all_icons, &icons));
        assert_eq!(2, all_icons.len());
        assert_eq!(Icon{x: 2, y: 2, href: "a".to_string(), poor: false},
                   get_best_icon(&mut all_icons, &icons));
        assert_eq!(4, all_icons.len());
    }

    // #[test]
    // fn get_mobile_icons_test() {
    //     let icon1 = Icon{x:1, y:1, href: "a".to_string(), poor: false};
    //     let icons = &[Icon{x:1, y:1, href: "a".to_string(), poor: false}];
    //     assert_eq!(
    //         Some(icon1),
    //         get_mobile_icons("a", Some(i
    //                                    clone()])));
    // }
}


