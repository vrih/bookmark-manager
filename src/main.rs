extern crate clap;
extern crate rbmlib;


//icon
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate select;


use clap::{App, Arg, SubCommand};
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::fs::OpenOptions;
use std::env;

use rbmlib::Bookmark;

mod icon;

fn list_bookmarks(path: &str) -> Result<(), io::Error>{
    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let b = Bookmark::new_from_line(l);
        println!("{}", b);
    }
    Ok(())
}

fn add_bookmark(path: &str, url: &str, title: &str, tags: &str) -> Result<(), io::Error>{
    let image_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(e) => panic!("Set RBM_BASE env")
    };
    
    let f = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();

    let b = Bookmark::new_from_input(String::from(url), String::from(title), String::from(tags));
    let c = b.output();
    writeln!(&f, "{}", c).unwrap();
    let mut fs_path:String = image_path;
    fs_path.push_str("/.bm.shots/");
    fs_path.push_str(&b.hash);
    fs_path.push_str(".png");
    println!("{}", fs_path);
    update_image(url, &fs_path);
    Ok(())
}

fn output_html(path: &str) -> Result<(), io::Error>{
    let mut bs: Vec<Bookmark> = Vec::new();

    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines(){
        let l = line.unwrap();
        let b = Bookmark::new_from_line(l);
        bs.push(b);
    }
    
    let a = rbmlib::html_output(bs);

    println!("{}", a);
    Ok(())
}

fn update_image(path: &str, fs_path: &str) -> Result<(), String>{
    let image_url = icon::get_image(path);
    let mut image_url:String = match image_url{
        Some(u) => u,
        None => return Err(String::from("No image url"))
    };
    
    println!("{}", image_url);

   // let dots = &image_url[..3];

    if &image_url[..3] == "../"{
        image_url = String::from(&image_url[3..]);
    }
    
    if &image_url[..2] == "//"{
        image_url.insert_str(0, "http:");
    }
    
    let slice = &image_url[..4];
    if slice == "http"{
        icon::download_media(&image_url, fs_path);
    } else {
        println!("{}", slice);
        let mut full_path: String = String::from(path);
        full_path.push_str(&image_url);
        println!("{}", full_path);
        icon::download_media(&full_path, fs_path);
    };
    
    Ok(())
}

fn refresh_image(label: &str){
    // refresh the iage for an existing bookmark
    println!("{}", label)
}

fn main() {
    let matches = App::new("Bookmark Manager")
        .version("1.0")
        .author("Daniel Bowman")
        .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .value_name("FILE")
             .help("Location of bookmarks file")
             .takes_value(true))
        .subcommand(SubCommand::with_name("add")
                    .arg(Arg::with_name("url")
                         .short("u")
                         .long("url")
                         .value_name("URL")
                         .help("Url to add")
                         .takes_value(true))
                    .arg(Arg::with_name("title")
                         .short("T")
                         .long("title")
                         .value_name("TITLE")
                         .help("URL title")
                         .takes_value(true))
                    .arg(Arg::with_name("taglist")
                         .short("t")
                         .long("taglist")
                         .value_name("TAGLIST")
                         .help("tags to add")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("html"))
        .subcommand(SubCommand::with_name("image")
                    .arg(Arg::with_name("label")
                         .short("l")
                         .long("label")
                         .value_name("LABEL")
                         .takes_value(true)))
        .get_matches();

    let file = matches.value_of("file").unwrap_or("test");

    if let Some(matches) = matches.subcommand_matches("add") {
        let url = matches.value_of("url").unwrap();
        let title = matches.value_of("title").unwrap_or("Default");
        let taglist = matches.value_of("taglist").unwrap_or("default");

        add_bookmark(file, url, title, taglist).unwrap()
            
    }
    
    if let Some(_) = matches.subcommand_matches("html") {
        output_html(file).unwrap();
    }

    
    if let Some(matches) = matches.subcommand_matches("image") {
        let label = matches.value_of("label").unwrap();
        refresh_image(&label);
    }
    //list_bookmarks(file).unwrap();
}
