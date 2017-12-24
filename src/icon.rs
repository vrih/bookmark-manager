use std::fs::File;

use select::document::Document;
use select::node::Node;
use select::predicate::*;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use hyper_tls::HttpsConnector;
use  hyper::header::Location;

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

pub fn get_image(url: &str) -> Option<String>{
    let core = Core::new();
    
    let mut core = match core {
        Ok(c) => c,
        Err(e) => panic!("Error setting up core")
    };

    let handle = core.handle();

    let httpsconnector = HttpsConnector::new(4, &handle);
    let httpsconnector = match httpsconnector {
        Ok(h) => h,
        Err(e) => panic!("Error setting up https connector: {:?}", e)
    };

    let client = Client::configure()
        .connector(httpsconnector)
        .build(&handle);

    let uri = url.parse();

    let uri = match uri {
        Ok(u) => u,
        Err(e) => panic!("Unable to parse URL")
    };

    let work = client.get(uri)
        .map_err(|_err| ())
        .and_then(|res| {
            // check for 200
            // if res.status().is_redirection() == true{
            //     let h = res.headers();
            //     let l = h.get::<Location>().unwrap();
            //     let b = get_image(&l);
            //     return Some(b)
            // } else {
            res.body()
                .concat2()
                .map_err(|_err| ())
                .map(|chunk| {
                    let v = chunk.to_vec();
                    let s = String::from_utf8_lossy(&v).to_string();
                    
                    let document = Document::from(&s[..]);
                    
                    let url = get_image_paths(&document);
                    let url = match url{
                        Some(s) => Some(s),
                        None => None
                    };
                    
                    return Some(url)
                }
                )
        }
                  
        
        );
    let c = core.run(work);
    let c = match c {
        Ok(d) => d,
        Err(e) => None
    };

    let c = match c{
        Some(d) => d,
        None => None
    };
    c
}


pub fn download_media(url: &str, fs_path: &str){
    let core = Core::new();
    println!("FN {}", url);
    let mut core = match core {
        Ok(c) => c,
        Err(e) => panic!("Error setting up core")
    };

    let handle = core.handle();

    let httpsconnector = HttpsConnector::new(4, &handle);
    let httpsconnector = match httpsconnector {
        Ok(h) => h,
        Err(e) => panic!("Error setting up https connector: {:?}", e)
    };
    
    let client = Client::configure()
        .connector(httpsconnector)
        .build(&handle);
        
    let uri = url.parse();

    let uri = match uri {
        Ok(u) => u,
        Err(e) => panic!("Unable to parse URL")
    };

    let work = client.get(uri)
        .map_err(|_err| ())
        .and_then(|res| {
            // check for 200
            println!("Response: {}", res.status());

            res.body()
                .concat2()
                .map_err(|_err| ())
                .map(|chunk| {
                    let v = chunk.to_vec();
                    return v
                })
                
        
        });
    let c = core.run(work);
    let c = match c {
        Ok(d) => d,
        Err(e) => panic!("{:?}", e)
    };

    println!("{}", fs_path);
    let mut f = File::create(fs_path).unwrap();
    f.write_all(&c);
}
