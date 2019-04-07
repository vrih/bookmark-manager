extern crate clap;
extern crate rbmlib;

//icon
extern crate select;
extern crate reqwest;
extern crate url;
extern crate chan;
extern crate serde_json;

use clap::{App, Arg, SubCommand};
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::fs::OpenOptions;
use std::env;

use rbmlib::Bookmark;

use std::thread;

static NTHREADS: i32 = 10;

mod icon;

fn list_bookmarks(path: &str) -> Result<(), io::Error>{
    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines() {
        match Bookmark::new_from_line(line?){
            Ok(b) => println!("{}", b),
            Err(_) => continue
        }
    }
    Ok(())
}

fn add_bookmark(path: &str, url: &str, title: &str, tags: &str, custom_image: &str) -> Result<(), reqwest::Error>{
    let f = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();

    let b = Bookmark::new_from_input(String::from(url), String::from(title), String::from(tags), String::from(custom_image));
    let c = b.output();
    writeln!(&f, "{}", c).unwrap();
    let fs_path = image_path(&b.hash);
    println!("{}", fs_path);
    update_image(url, &fs_path)
}

fn image_path(hash: &str) -> String{
    let image_path = env::var("RBM_BASE").expect("Set RBM_BASE env");

    format!("{}/.bm.shots/{}.png", &image_path, hash)
}

fn output_html(path: &str) -> Result<(), io::Error>{
    let mut bs: Vec<Bookmark> = Vec::new();
    
    let directory_path = env::var("RBM_BASE").expect("Set RBM_BASE env");

    let directory_path = format!("{}/bm.html", directory_path);

    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines(){
        match Bookmark::new_from_line(line?){
            Ok(b) => bs.push(b),
            Err(_) => continue
        };
    }
    
    let fo = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(directory_path)
        .unwrap();
    
    let a = rbmlib::html_output(bs);
    write!(&fo, "{}", a)
}

fn update_image(path: &str, fs_path: &str) -> Result<(), reqwest::Error>{
    icon::download_image(path, fs_path)
}

fn refresh_all_images(path: &str) -> Result<(), io::Error>{
    let r = {
        let (s, r) = chan::sync(0);

        let f = try!(File::open(path));
        thread::spawn(move || 
                      {
                          let file = BufReader::new(&f);
                          for line in file.lines() {
                              let l = line.unwrap();
                              let b = match Bookmark::new_from_line(l){
                                  Ok(b) => b,
                                  Err(_) => continue
                              };
                              s.send(b);
                          }
                      });
        r
    };

    let wg = chan::WaitGroup::new();
    for _ in 0..NTHREADS {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        wg.add(1);
        let wg = wg.clone();
        let r = r.clone();
        thread::spawn(move || {
            for bm in r{
                if bm.custom_image.len() > 0 {
                    continue
                }
                match update_image(&bm.url, &image_path(&bm.hash)){
                    Ok(_) => println!("Updated: {}", &bm.title),
                    Err(e) => {
                        println!("{:?}", e);
                        println!("Error updating {}", &bm.title)
                    }
                };   
            }
            wg.done();
        });

    }
    wg.wait();
        Ok(())
}

fn refresh_image(path: &str, label: &str) -> Result<(), io::Error>{
    // refresh the iage for an existing bookmark

    let f = try!(File::open(path));
    let file = BufReader::new(&f);
    for line in file.lines() {
        let b = match Bookmark::new_from_line(line?){
            Ok(b) => b,
            Err(_) => continue
        };
        if b.label == label{
            match update_image(&b.url, &image_path(&b.hash)){
                Ok(_) => return Ok(()),
                Err(_) => println!("Unable to refresh image")
            };
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
                         .takes_value(true))
                    .arg(Arg::with_name("custom_image")
                         .short("c")
                         .long("custom_image")
                         .value_name("CUSTOM_IMAGE")
                         .help("custom_image")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("html"))
        .subcommand(SubCommand::with_name("image")
                    .arg(Arg::with_name("all")
                         .short("a")
                         .long("all")
                         .takes_value(false))
                    .arg(Arg::with_name("label")
                         .short("l")
                         .long("label")
                         .value_name("LABEL")
                         .takes_value(true)))
        .get_matches();


    let file_env = env::var("RBM_BASE").expect("Set RBM_BASE env");
    
    let default_file_path = format!("{}/bm.lnk", file_env);
    
    let file = matches.value_of("file").unwrap_or(&default_file_path);
        
    if let Some(matches) = matches.subcommand_matches("add") {
        let url = matches.value_of("url").unwrap();
        let title = matches.value_of("title").unwrap_or("Default");
        let taglist = matches.value_of("taglist").unwrap_or("default");
        let custom_image = matches.value_of("custom_image").unwrap_or("");

        add_bookmark(file, url, title, taglist, custom_image).unwrap();
        output_html(file).unwrap();
            
    }
    
    if matches.subcommand_matches("list").is_some() {
        list_bookmarks(file).unwrap();
    }
    if matches.subcommand_matches("html").is_some() {
        output_html(file).unwrap();
    }

    
    if let Some(matches) = matches.subcommand_matches("image") {
        match matches.values_of("all"){
            Some(_) => refresh_all_images(file).unwrap(),
            _ => {
                let label = matches.value_of("label").unwrap();
                refresh_image(file, label).unwrap();
            }
        }
    }
}
