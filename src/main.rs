extern crate clap;
extern crate rbmlib;

//icon
extern crate select;
extern crate reqwest;
extern crate url;

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
    let f = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();

    let b = Bookmark::new_from_input(String::from(url), String::from(title), String::from(tags));
    let c = b.output();
    writeln!(&f, "{}", c).unwrap();
    let fs_path = image_path(&b.hash);
    println!("{}", fs_path);
    update_image(url, &fs_path)
}

fn image_path(hash: &str) -> String{
    let image_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(_e) => panic!("Set RBM_BASE env")
    };

    format!("{}/.bm.shots/{}.png", &image_path, hash)
}

fn output_html(path: &str) -> Result<(), io::Error>{
    let mut bs: Vec<Bookmark> = Vec::new();
    
    let directory_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(_e) => panic!("Set RBM_BASE env")
    };

    let directory_path = format!("{}/bm.html", directory_path);

    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines(){
        let l = line.unwrap();
        let b = Bookmark::new_from_line(l);
        bs.push(b);
    }
    
    let fo = OpenOptions::new()
        .write(true)
        .create(true)
        .open(directory_path)
        .unwrap();
    
    let a = rbmlib::html_output(bs);
    write!(&fo, "{}", a)
}

fn update_image(path: &str, fs_path: &str) -> Result<(), io::Error>{
    icon::download_image(path, fs_path)
}

fn refresh_image(path: &str, label: &str) -> Result<(), io::Error>{
    // refresh the iage for an existing bookmark

    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let b = Bookmark::new_from_line(l);
        if b.label == label{
            return update_image(&b.url, &image_path(&b.hash))
        }
    }
    // TODO: This should be an error, not OK
    Ok(())
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
        .subcommand(SubCommand::with_name("list"))
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


    let file_env = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(_e) => panic!("Set RBM_BASE env")
    };
    
    let default_file_path = format!("{}/bm.lnk", file_env);
    
    let file = matches.value_of("file").unwrap_or(&default_file_path);
    
    
    if let Some(matches) = matches.subcommand_matches("add") {
        let url = matches.value_of("url").unwrap();
        let title = matches.value_of("title").unwrap_or("Default");
        let taglist = matches.value_of("taglist").unwrap_or("default");

        add_bookmark(file, url, title, taglist).unwrap();
        output_html(file).unwrap();
            
    }
    
    if let Some(_) = matches.subcommand_matches("list") {
        list_bookmarks(file).unwrap();
    }
    if let Some(_) = matches.subcommand_matches("html") {
        output_html(file).unwrap();
    }

    
    if let Some(matches) = matches.subcommand_matches("image") {
        let label = matches.value_of("label").unwrap();
        refresh_image(&file, &label).unwrap();
    }
    //list_bookmarks(file).unwrap();
}
