extern crate comrak;
extern crate regex;
extern crate rss;
extern crate time;
extern crate dotenv;
#[macro_use]
extern crate rouille;
extern crate chrono;

use dotenv::dotenv;
use std::env;
use comrak::{markdown_to_html, ComrakOptions};
use std::fs;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use regex::Regex;
use rss::ChannelBuilder;
use chrono::prelude::*;

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, PartialEq)]
struct Article {
    title: String,
    url: String
}

#[derive(Debug)]
struct Shared {
    tags: HashMap<String, Vec<Article>>
}

struct Metadata {
    title: String,
    published: String,
    date: String,
    description: String,
    image: String,
    tags: Vec<String>,
    markdown: String,
    output_file: PathBuf,
    output_html: String
}

fn post_can_be_parsed(status: &str) -> bool {
    status.eq("true") || status.eq("guest") || status.eq("private")
}

fn post_can_be_published(status: &str) -> bool {
    status.eq("true") || status.eq("guest")
}

fn load_template(folder: &str) -> std::io::Result<String> {
    let template = if folder.starts_with(".") {
        "posts"
    } else {
        folder
    };
    let mut f_template = File::open(format!("./templates/{}.template.html", template))?;
    let mut template_source = String::new();
    let _result = f_template.read_to_string(&mut template_source)?;
    Ok(template_source)
}

fn for_each_extension<F: Fn(&mut Shared, &Path) -> Option<Metadata>>(extension: &str, folder: &str, shared: &mut Shared, func: F) -> Vec<Metadata> {
    let mut posts: Vec<Metadata> = vec![];
    let paths = fs::read_dir(folder).unwrap();
    for entry in paths {
        let entry = entry.unwrap().path();
        let path = entry.as_path();
        if let Some(ext) = path.extension() {
            if ext == extension {
                if let Some(post) = func(shared, path) {
                    posts.push(post);
                }
            }
        }
    }
    posts
}

fn generate_tags(text: &str, tags: &Vec<String>) -> String {
    //dotenv().ok();
    let output: Vec<String> = tags.into_iter().map(|t| format!("<a class='topic-tag' href='/tags/{}.html'>{}</a>", t, t)).collect();
    let inner_html = output.as_slice().join("");
    format!("<div class='other-tags'><b>{}Tags:</b> {}</div>", text, inner_html)
}

fn generate_related_post(shared: Option<&Shared>, tags: &Vec<String>, current: String) -> String {
    let mut posts: Vec<Article> = vec![];
    if let Some(shared) = shared {
        for tag in tags {
            if let Some(posts_in_tag) = shared.tags.get(tag) {
                posts.extend(posts_in_tag.iter().cloned());
            }
        }
    }
    posts.dedup();
    if let Some(index) = (&posts).into_iter().position(|x| x.title == current) {
        posts.remove(index);
    }
    let total = posts.len();
    let limit = if total < 5 { total } else { 5 };
    let output: Vec<String> = posts.into_iter().map(|t| format!("<li><a class='related-post' href='/posts/{}'>{}</a></li>", t.url, t.title)).collect();
    format!("<ul class='related-posts-list'>{}</ul>", (&output[..limit]).join(""))
}

fn generate_meta(post: &Metadata) -> String {
    let domain_name = env::var("DOMAIN_NAME").unwrap();
    let default_img = format!("{}/img/default.jpg", domain_name);
    let img = match post.image.as_ref() {
        "" => &default_img[..],
        _  => &post.image
    };
    format!("<meta property='og:image' content='{}'>", img)
}

fn apply_template(template: &str, post: &Metadata, tag_text: &str, related_posts: Option<&Shared>) -> String {
    //dotenv().ok();
    let mut options = ComrakOptions::default();
    options.ext_strikethrough = true;
    options.ext_table = true;
    options.ext_autolink = true;
    options.ext_header_ids = Some("".to_string());
    options.ext_footnotes = true;
    let parsed = markdown_to_html(&post.markdown, &options);
    let file_name = post.output_file.file_name().unwrap().to_str().unwrap();
    let html =
        &template
        .replace("{%content%}", &parsed)
        .replace("{%title%}", &post.title)
        .replace("{%meta%}", &generate_meta(&post))
        .replace("{%hash%}", "")
        .replace("{%tags%}", &generate_tags(tag_text, &post.tags))
        .replace("{%related%}", &generate_related_post(related_posts, &post.tags, (&post.title).to_string()))
        .replace("{%postslug%}", &file_name.replace(".html", ""))
        .replace("{%posturl%}", &format!("{}/posts/{}", env::var("DOMAIN_NAME").unwrap(), file_name));
    format!("{}", html)
}

fn save_as_html(html: &str, output_path: &PathBuf) -> std::io::Result<bool> {
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
        published: format!("false"),
        date: format!(""),
        description: format!(""),
        image: format!(""),
        tags: vec![],
        markdown: format!(""),
        output_file: path.with_extension("html"),
        output_html: format!("")
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
                metadata.published = line.replace("published: ", "");
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

fn generate_index_page(posts: &Vec<Metadata>) {
    //dotenv().ok();
    let date_format = env::var("DATE_FORMAT").unwrap();
    let domain_name = env::var("DOMAIN_NAME").unwrap();
    if let Ok(template) = load_template("index") {
        let html: Vec<String> = posts.into_iter().map(|p| {
            let file_name = p.output_file.file_name().unwrap().to_str().unwrap();
            let post_date = Utc.datetime_from_str(&p.date, TIME_FORMAT).unwrap();
            let post_date_text = post_date.format(&date_format);
            let tag_list = &p.tags.join(", ");
            let guest_tag = if p.published.eq("guest") {
                "<span class='guest-post'>Guest Post</span>"
            } else {
                ""
            };
            format!("<div class='home-list-item'><span class='home-date-indicator'>{}</span>{}{}<br/><a href='/posts/{}'>{}</a></div>", post_date_text, guest_tag, tag_list, file_name, p.title)
        }).collect();
        let markdown = html.join("\n");
        let mut post = Metadata {
            title: "Index".to_string(),
            published: format!("true"),
            date: "".to_string(),
            description: "".to_string(),
            image: "".to_string(),
            tags: vec![],
            markdown: markdown,
            output_file: PathBuf::from("./index.html"),
            output_html: format!("")
        };
        post.output_html = apply_template(&template, &post, "", None);
        let _ = save_as_html(&post.output_html, &PathBuf::from("./index.html"));
    }
}

fn generate_tags_page(tags: &HashMap<String, Vec<Article>>) {
    //dotenv().ok();
    if let Ok(template) = load_template("tags") {
        for (key, value) in tags.into_iter() {
            println!("{} - {:?}", key, value);
            let post_list: Vec<String> = value.into_iter().map(|a| format!("- [{}](/posts/{})", a.title, a.url)).collect();
            let markdown = post_list.join("\n");
            let tags_except_key: Vec<String> = tags.keys().into_iter().filter(|k| *k != key).map(|k| format!("{}", k)).collect();
            let post = Metadata {
                title: format!("{}", &key),
                published: format!("true"),
                date: "".to_string(),
                description: "".to_string(),
                image: "".to_string(),
                tags: Vec::from(tags_except_key),
                markdown: markdown,
                output_file: PathBuf::from(&format!("./tags/{}.html", &key)),
                output_html: format!("")
            };
            let output_html = apply_template(&template, &post, "Other ", None);
            let _ = save_as_html(&output_html, &post.output_file);
        }
    }
}

fn generate_rss_feed(posts: &Vec<Metadata>) {
    //dotenv().ok();
    let mut channel = ChannelBuilder::default()
        .title(env::var("RSS_TITLE").unwrap())
        .link(env::var("DOMAIN_NAME").unwrap())
        .description(env::var("RSS_DESCRIPTION").unwrap())
        .build()
        .unwrap();

    let mut items: Vec<rss::Item> = vec![];
    for post in posts {
        let file_name = post.output_file.file_name().unwrap().to_str().unwrap();
        let post_date = Utc.datetime_from_str(&post.date, TIME_FORMAT).unwrap();
        let post_date_text = post_date.to_rfc2822();
        println!("{:?}  {:?}  {:?}", post_date, post_date_text, &post.date);
        let mut item = rss::Item::default();
        let link = format!("{}/posts/{}", env::var("DOMAIN_NAME").unwrap(), file_name);
        let mut guid = rss::Guid::default();
        guid.set_value(link.clone());
        item.set_title(format!("{}", &post.title));
        item.set_guid(guid);
        item.set_link(link.clone());
        item.set_description(format!("{:?}", &post.description));
        item.set_pub_date(post_date_text);
        items.push(item);
    }
    channel.set_items(items);

    let mut output_file = File::create("./rss.xml").unwrap();
    output_file.write_all(channel.to_string().as_bytes());
}

fn parse_post(template: &str, shared: &Shared, path: &Path, force: bool) -> Option<Metadata> {
    let emoji_regex: Regex = Regex::new(r#" :\b([a-z\-]+)\b:"#).unwrap();
    let mut post = parse_metadata(path);
    if post_can_be_parsed(&post.published) || force {
        println!("Title: {}\nTags: {:?}\nFile: {:?}\n", post.title, post.tags, post.output_file.file_name());
        // Parse cover
        post.markdown = custom_parser(&post.markdown, |src| src
                                      .replace("<cover>", "<div class='cover' style='background-image: url(")
                                      .replace("</cover>", ")'></div><div class='cover-holder'></div>"));
        // Parse Emoji
        post.markdown = custom_parser(&post.markdown, |src| emoji_regex.replace_all(src, " <i class='em em-$1'></i>").into_owned());
        // Parse math
        post.markdown = custom_parser(&post.markdown, |src| src.replace("<math>", "<pre class='math'>$$").replace("</math>", "$$</pre>"));
        // Parse video
        post.markdown = custom_parser(&post.markdown, |src| src.replace("<animate>", "<video style='max-width: 800px; margin-left: -140px' autoplay loop><source type='video/mp4' src='").replace("</animate>", "'></source></video>"));

        let output_html = apply_template(&template, &post, "", Some(&shared));
        post.output_html = output_html;
        return Some(post);
    }
    None
}

fn main() {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let mut folder = ".";
    if args.len() > 1 {
        let param = &args[1];

        if (param != "preview") {
            folder = param;
            // Generator mode

            // TODO: Default template should come from config file as well
            if let Ok(template) = load_template(folder) {
                let mut shared = Shared { tags: HashMap::new() };

                let _ = for_each_extension("md", folder, &mut shared, move |shared, path| {
                    let mut post = parse_metadata(path);
                    if post_can_be_parsed(&post.published) {
                        println!("Title: {}\nTags: {:?}\nFile: {:?}\n", post.title, post.tags, post.output_file.file_name());
                        // Parse tags
                        for tag in &post.tags {
                            if post_can_be_published(&post.published) {
                                let find_tag = format!("{}", tag);
                                if !shared.tags.contains_key(&find_tag) {
                                    shared.tags.insert(format!("{}", tag), vec![]);
                                }
                                let tag_posts = shared.tags.get_mut(&format!("{}", tag)).unwrap();
                                tag_posts.push(
                                    Article {
                                        title: format!("{}", &post.title),
                                        url: format!("{}", post.output_file.file_name().unwrap().to_str().unwrap())
                                    }
                                );
                            }
                        }
                    }
                    None
                });


                let mut posts =
                    for_each_extension("md", folder, &mut shared, move |shared, path| {
                        if let Some(post) = parse_post(&template, shared, path, false) {
                            let _save_result = save_as_html(&post.output_html, &post.output_file);
                            if post_can_be_published(&post.published) {
                                return Some(post);
                            } else {
                                return None;
                            }
                        }
                        None
                    });

                posts.sort_by(|a, b| {
                    let ta = Utc.datetime_from_str(&a.date, TIME_FORMAT).unwrap();
                    let tb = Utc.datetime_from_str(&b.date, TIME_FORMAT).unwrap();
                    tb.cmp(&ta)
                });

                println!("Total {} posts", posts.len());
                generate_index_page(&posts);

                println!("Tags: {:?}", shared.tags);
                generate_tags_page(&shared.tags);

                generate_rss_feed(&posts);
            }

        } else {
            // Preview mode
            println!("Preview server running at :3123");

            rouille::start_server("localhost:3123", move |request| {
                {
                    let response = rouille::match_assets(&request, ".");
                    println!("MATCHING {:?}", response);
                    if response.is_success() {
                        return response;
                    }
                }
                router!(request,
                    (GET) (/view/{file_name: String}) => {
                        if let Ok(template) = load_template("preview") {
                            let mut shared = Shared { tags: HashMap::new() };
                            let path = PathBuf::from(format!("./posts/{}.md", file_name));
                            let abs_path = fs::canonicalize(&path).unwrap();
                            if let Some(post) = parse_post(&template, &shared, &PathBuf::from(abs_path), true) {
                                let output = post.output_html.replace("\"img", "\"/posts/img").to_string();
                                return rouille::Response::html(output); 
                            }
                        }
                        rouille::Response::empty_404()
                    },

                    _ => rouille::Response::empty_404()
                )
            });
        }
    }
}
