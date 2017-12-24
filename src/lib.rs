extern crate crypto;
extern crate time;

use crypto::md5::Md5;
use crypto::digest::Digest;
use std::fmt;
use std::env;

use std::string::String;

use std::io::prelude::*;
use std::fs::File;

#[derive(PartialEq, Debug)]
pub struct Bookmark {
    pub hash: String,
    created_at: time::Tm,
    pub label: String,
    pub url: String,
    title: String,
    tags: String,
    //image: &'a str,
}

const ISO_TIME_DATE: &str = "%Y-%m-%dT%H:%M:%SZ";


impl Bookmark {
    pub fn new_from_line(line: String) -> Bookmark {
        let fields: Vec<&str> = line.split("|").collect();
        let hash = String::from(fields[0]);
        let created_at = time::strptime(fields[1], ISO_TIME_DATE).unwrap();
        let label = String::from(fields[2]);
        let url = String::from(fields[3]);
        let title = String::from(fields[4]);
        let tags = String::from(fields[5]);
        Bookmark{hash, created_at, label, url, title, tags}
    }

    pub fn new_from_input(url: String, title: String, tags: String) -> Bookmark {
        let mut hasher = Md5::new();
        hasher.input_str(url.as_str());
        let hash = hasher.result_str();

        let created_at = time::now();
        // how to create label
        let label = hash[..5].to_string();
        let tags = tags;
        Bookmark{hash, created_at, label, url, title, tags}
    }

    pub fn output(&self) -> String {
        let s = format!("{}|{}|:{}|{}|{}|{}", self.hash, time::strftime(ISO_TIME_DATE, &self.created_at).unwrap(),
                        self.label, self.url, self.title, self.tags);
        return s
    }
}

impl fmt::Display for Bookmark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{} --> {} [{}]\n{} - Added: {}\n",
               self.label, self.title, self.tags, self.url,
               time::strftime(ISO_TIME_DATE, &self.created_at).unwrap())

    }
}

pub fn html_output(bookmarks: Vec<Bookmark>) -> String {
    let image_path = match env::var("RBM_BASE"){
        Ok(a) => a,
        Err(e) => panic!("Set RBM_BASE env")
    };
    
    let mut file = File::open(format!("{}/.template.html", &image_path)).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");
    
    let mut buffer = String::new();
    // convert to map
    for bm in bookmarks {
        let tagstring = bm.tags.replace(",", " ");
        let bs = format!("<div class=\"bm {}\"><a href='{}'><img src='.bm.shots/{}.png'></a><p>{}</p></div>",
                         tagstring, bm.url, bm.hash, bm.title);
        buffer.push_str(&bs);
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
        tags: String::from("tag1,tag2")}, Bookmark::new_from_line(line))
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


