extern crate comrak;
extern crate regex;

use comrak::{markdown_to_html, ComrakOptions};
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use regex::Regex;

struct Shared {
    tags: HashMap<String, Vec<String>>
}

struct Metadata {
    title: String,
    published: bool,
    date: String,
    description: String,
    image: String,
    tags: Vec<String>,
    markdown: String,
    output_file: PathBuf
}

fn load_template(folder: &str) -> std::io::Result<String> {
    let mut f_template = File::open(format!("./templates/{}.template.html", folder))?;
    let mut template_source = String::new();
    let _result = f_template.read_to_string(&mut template_source)?;
    Ok(template_source)
}

fn for_each_extension<F: Fn(&mut Shared, &Path)>(extension: &str, folder: &str, shared: &mut Shared, func: F) {
    let paths = fs::read_dir(folder).unwrap();
    for entry in paths {
        let entry = entry.unwrap().path();
        let path = entry.as_path();
        if let Some(ext) = path.extension() {
            if ext == extension {
                func(shared, path);
            }
        }
    }
}

fn apply_template(template: &str, post: &Metadata) -> String {
    let parsed = markdown_to_html(&post.markdown, &ComrakOptions::default());
    let file_name = post.output_file.file_name().unwrap().to_str().unwrap();
    let html =
        &template
        .replace("{%content%}", &parsed)
        .replace("{%title%}", &post.title)
        .replace("{%meta%}", "")
        .replace("{%hash%}", "")
        .replace("{%postslug%}", &file_name.replace(".html", ""))
        .replace("{%posturl%}", &format!("https://thefullsnack.com/posts/{}", file_name));
    format!("{}", html)
}

fn save_as_html(html: &str, output_path: PathBuf) -> std::io::Result<bool> {
    let mut output_file = File::create(output_path).unwrap();
    output_file.write_all(html.as_bytes())?;
    Ok(true)
}

fn custom_parser<F: Fn(&str) -> String>(markdown: &str, func: F) -> String {
    func(markdown)
}

fn parse_metadata(path: &Path) -> Metadata {
    let mut metadata = Metadata {
        title: format!(""),
        published: false,
        date: format!(""),
        description: format!(""),
        image: format!(""),
        tags: vec![],
        markdown: format!(""),
        output_file: path.with_extension("html")
    };

    let mut line_count = 0;
    let f = File::open(path).unwrap();
    for line in BufReader::new(f).lines() {
        let line = line.unwrap();
        line_count += 1;
        if line_count > 8 {
            metadata.markdown = format!("{}\r\n{}", metadata.markdown, line);
        } else {
            if line.starts_with("title: ") {
                metadata.title = line.replace("title: ", "");
            }
            else if line.starts_with("published: ") {
                metadata.published = line.replace("published: ", "").eq("true");
            }
            else if line.starts_with("date: ") {
                metadata.date = line.replace("date: ", "");
            }
            else if line.starts_with("tags: ") {
                metadata.tags = line.replace("tags: ", "").split(", ").map(|s| s.to_string()).collect();
            }
            else if line.starts_with("description: ") {
                metadata.description = line.replace("description: ", "");
            }
            else if line.starts_with("image: ") {
                metadata.image = line.replace("image: ", "");
            }
        }
    }

    metadata
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let folder = &args[1];
    let emoji_regex: Regex = Regex::new(r#" :\b([a-z\-]+)\b:"#).unwrap();
    let ref_regex: Regex = Regex::new(r#"<ref>([0-9]+)</ref>"#).unwrap();
    let cite_regex: Regex = Regex::new(r#"<cite id="([0-9]+)">(.*)</cite>"#).unwrap();

    let template = load_template(folder).unwrap();

    let mut shared = Shared { tags: HashMap::new() };

    for_each_extension("md", folder, &mut shared, move |shared, path| {
        let mut post = parse_metadata(path);
        if post.published {
            println!("Title: {}\nTags: {:?}\nFile: {:?}\n", post.title, post.tags, post.output_file.file_name());
            // Parse tags
            for tag in &post.tags {
                let find_tag = format!("{}", tag);
                if !shared.tags.contains_key(&find_tag) {
                    shared.tags.insert(format!("{}", tag), vec![]);
                }
                let mut tag_posts = shared.tags.get_mut(&format!("{}", tag)).unwrap();
                tag_posts.push(format!("{}", post.output_file.file_name().unwrap().to_str().unwrap()));
            }
            // Parse cover
            post.markdown = custom_parser(&post.markdown, |src| src
                                           .replace("<cover>", "<div class='cover' style='background-image: url(")
                                           .replace("</cover>", ")'></div><div class='cover-holder'></div>"));
            // Parse Emoji
            post.markdown = custom_parser(&post.markdown, |src| emoji_regex.replace_all(src, " <i class='em em-$1'></i>").into_owned());
            // Parse math
            post.markdown = custom_parser(&post.markdown, |src| src.replace("<math>", "<pre class='math'>$$").replace("</math>", "$$</pre>"));
            // Parse refs
            post.markdown = custom_parser(&post.markdown, |src| ref_regex.replace_all(src, r#"<sup><a id='reference-link-$1' href='#reference-area-$1'>\[$1\]</a></sup>"#).into_owned());
            post.markdown = custom_parser(&post.markdown, |src| cite_regex.replace_all(src, r#"<div id='reference-area-$1'>\[$1\] $2 <sup alt='Quay trở lên trên'><a href='#reference-link-$1'>↑ return</a></sup></div>"#).into_owned());

            let output_html = apply_template(&template, &post);
            let _save_result = save_as_html(&output_html, post.output_file);
        }
    });

    println!("Tags: {:?}", shared.tags);
}
