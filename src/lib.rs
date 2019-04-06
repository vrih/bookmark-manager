extern crate crypto;
extern crate time;
extern crate colored;

use crypto::md5::Md5;
use crypto::digest::Digest;
use std::fmt;
use std::env;

use std::path::Path;
use std::string::String;

use std::io::prelude::*;
use std::fs::File;

use colored::*;

#[derive(PartialEq, Debug)]
pub struct Bookmark {
    pub hash: String,
    created_at: time::Tm,
    pub label: String,
    pub url: String,
    pub title: String,
    tags: String,
    custom_image: String,
    //image: &'a str,
}

const ISO_TIME_DATE: &str = "%Y-%m-%dT%H:%M:%SZ";

impl Bookmark {
    pub fn new_from_line(line: String) -> Result<Bookmark, String> {
        let fields: Vec<&str> = line.split("|").collect();

        if fields.len() < 5{
            return Err(String::from("Not enough fields in line"))
        }
        
        let hash = String::from(fields[0]);
        let created_at = time::strptime(fields[1], ISO_TIME_DATE).unwrap();
        let label = String::from(fields[2]);
        let url = String::from(fields[3]);
        let title = String::from(fields[4]);
        let tags = String::from(fields[5]);
        let custom_image = String::from(fields[6]);
        Ok(Bookmark{hash, created_at, label, url, title, tags, custom_image})
    }

    pub fn new_from_input(url: String, title: String, tags: String, custom_image: String) -> Bookmark {
        let mut hasher = Md5::new();
        hasher.input_str(url.as_str());
        let hash = hasher.result_str();

        let created_at = time::now();
        // how to create label
        let label = hash[..5].to_string();
        let tags = tags;
        let custom_image = custom_image;
        Bookmark{hash, created_at, label, url, title, tags, custom_image}
    }

    pub fn output(&self) -> String {
        let s = format!("{}|{}|:{}|{}|{}|{}", self.hash, time::strftime(ISO_TIME_DATE, &self.created_at).unwrap(),
                        self.label, self.url, self.title, self.tags);
        return s
    }
}

impl fmt::Display for Bookmark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{} {} {} [{}]\n{} — {}{}\n",
               self.label.bold().bright_black(),
               "→".dimmed(),
               self.title.white(),
               self.tags.bold(),
               self.url.underline().dimmed(),
               "Added: ".bright_black(),
               time::strftime(ISO_TIME_DATE, &self.created_at).unwrap().bright_black())
    }
}

fn image_exists(filename: &str) -> Option<String>{
    let base_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(_e) => panic!("Set RBM_BASE env")
    };

    if filename.len() == 0 {
        return None
    };
    
    let file_endings = ["", ".svg", ".png"];
    
    for element in file_endings.iter() {
        let tail = format!("{}{}", filename, element);
        let path = format!("{}/.bm.shots/{}", base_path ,tail);
        if Path::new(&path).exists() {
            return Some(tail);
        }
    }
    return None;
}

pub fn html_output(bookmarks: Vec<Bookmark>) -> String {
    let image_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(_e) => panic!("Set RBM_BASE env")
    };
    
    let mut file = File::open(format!("{}/.template.html", &image_path)).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");
    
    let mut buffer = String::new();
    // convert to map
    for bm in bookmarks {
        let tagstring = bm.tags.replace(",", " ");
        let image_path = image_exists(&bm.custom_image).or(image_exists(&bm.hash));
        let icon_tag = match image_path{
            Some(path) =>  format!("<div class=\"bm {}\"><a href='{}'><img src='.bm.shots/{}'><p>{}</p></a></div>",
                                  tagstring, bm.url, path, bm.title),
            None => format!("<div class=\"bm noimage {}\"><a href='{}'><div class=\"letter\">{}</div><p>{}</p></a></div>",
                           tagstring, bm.url, bm.title.chars().next().expect("No title"), bm.title)};
        buffer.push_str(&icon_tag);
    }

    let html = contents.replace("//REPLACE//", &buffer);
    return html
}

#[test]
fn line_to_file_test() {
    let line = String::from("a123|2017-12-18T11:46:29Z|:5|https://www.example.com/|Example|tag1,tag2");
    
    assert_eq!(Bookmark{
        hash: String::from("a123"),
        created_at: time::strptime("2017-12-18T11:46:29Z", ISO_TIME_DATE).unwrap(),
        label: String::from(":5"),
        url: String::from("https://www.example.com/"),
        title: String::from("Example"),
        tags: String::from("tag1,tag2")}, Bookmark::new_from_line(line).unwrap())
}

#[test]
fn blank_line_to_file_test() {
    let line = String::from("");
    
    assert!(Bookmark::new_from_line(line).is_err())
}


// Disabled until I can work out how to mock time
// #[test]
// fn bookmark_from_input(){
//     assert_eq!(Bookmark{
//         hash: String::from("change me"),
//         // how to mock time properly
//         created_at: time::now(),
//         label: String::from("change me"),
//         url: String::from("https://www.example.com/"),
//         title: String::from("Example"),
//         tags: String::from("tag1,tag2")},
//                Bookmark::new_from_input(String::from("https://www.example.com/"),
//                                         String::from("Example"), String::from("tag1,tag2"))
//     )
// }


