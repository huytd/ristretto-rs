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
use std::sync::{Arc, RwLock};
use regex::Regex;
use rss::ChannelBuilder;
use chrono::prelude::*;

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
struct Cache<T> {
    last_cached: std::time::SystemTime,
    time_to_live: std::time::Duration,
    data: T
}

impl<T: Clone> Cache<T> {
    fn new(content: T, ttl: std::time::Duration) -> Cache<T> {
        Cache {
            last_cached: std::time::SystemTime::now(),
            time_to_live: ttl,
            data: content
        }
    }

    fn is_expired(&self) -> bool {
        self.last_cached.elapsed().unwrap() >= self.time_to_live
    }

    fn update(&mut self, updated_content: T) {
        println!("Cache expired. Updated at {:?}", std::time::SystemTime::now());
        self.last_cached = std::time::SystemTime::now();
        self.data = updated_content;
    }

    fn get(&self) -> T {
        self.data.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Article {
    title: String,
    url: String
}

#[derive(Debug)]
struct Shared {
    tags: HashMap<String, Vec<Article>>
}

#[derive(Debug, Clone)]
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

fn parse_post(template: &str, shared: &Shared, path: &Path, force: bool) -> Option<Metadata> {
    let emoji_regex: Regex = Regex::new(r#" :\b([a-z\-_]+)\b:"#).unwrap();
    let mut post = parse_metadata(path);
    if post_can_be_parsed(&post.published) || force {
        //println!("Title: {}\nTags: {:?}\nFile: {:?}\n", post.title, post.tags, post.output_file.file_name());
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

fn get_posts() -> Vec<Metadata> {
    let mut shared = Shared { tags: HashMap::new() };
    let mut posts = for_each_extension("md", "./posts", &mut shared, move |shared, path| {
        //println!(">>>> {}", &path.display());
        if let Some(post) = parse_post("", shared, path, false) {
            //println!("Can be pulbisehd {} {}", &post.title, &post.published);
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

    posts
}

fn response_index(cache: Arc<RwLock<Cache<Vec<Metadata>>>>, template: &str) -> rouille::Response {
    let has_posts: Option<Vec<Metadata>> = {
        if let Some(cached) = cache.read().ok() {
            if cached.is_expired() {
                None
            } else {
                Some(cached.get())
            }
        } else {
            None
        }
    };
    let posts = if let Some(post) = has_posts {
        post
    } else {
        if let Some(mut writer) = cache.write().ok() {
            writer.update(get_posts());
            writer.get()
        } else {
            vec![]
        }
    };

    let date_format = env::var("DATE_FORMAT").unwrap();
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
    return rouille::Response::html(post.output_html);
}

fn response_view(file_name: String, template: &str) -> rouille::Response {
    let shared = Shared { tags: HashMap::new() };
    let file_name_without_ext = file_name.replace(".html", "");
    let path = PathBuf::from(format!("./posts/{}.md", file_name_without_ext));
    let abs_path = fs::canonicalize(&path).unwrap();
    if let Some(post) = parse_post(&template, &shared, &PathBuf::from(abs_path), true) {
        let output = post.output_html.replace("\"img", "\"/posts/img").to_string();
        return rouille::Response::html(output);
    }
    rouille::Response::empty_404()
}

fn response_tag(cache: Arc<RwLock<Cache<Vec<Metadata>>>>, tag_name: String, template: &str) -> rouille::Response {
    let has_posts: Option<Vec<Metadata>> = {
        if let Some(cached) = cache.read().ok() {
            if cached.is_expired() {
                None
            } else {
                Some(cached.get())
            }
        } else {
            None
        }
    };
    let posts = if let Some(post) = has_posts {
        post
    } else {
        if let Some(mut writer) = cache.write().ok() {
            writer.update(get_posts());
            writer.get()
        } else {
            vec![]
        }
    };
    let tag = tag_name.replace(".html", "");
    let posts: Vec<Metadata> = posts.into_iter().filter(|p| p.tags.contains(&tag)).collect();

    let date_format = env::var("DATE_FORMAT").unwrap();
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
        output_file: PathBuf::from("./tags.html"),
        output_html: format!("")
    };
    post.output_html = apply_template(&template, &post, "", None);
    return rouille::Response::html(post.output_html);
}

fn response_rss(cache: Arc<RwLock<Cache<Vec<Metadata>>>>) -> rouille::Response {
    let has_posts: Option<Vec<Metadata>> = {
        if let Some(cached) = cache.read().ok() {
            if cached.is_expired() {
                None
            } else {
                Some(cached.get())
            }
        } else {
            None
        }
    };
    let posts = if let Some(post) = has_posts {
        post
    } else {
        if let Some(mut writer) = cache.write().ok() {
            writer.update(get_posts());
            writer.get()
        } else {
            vec![]
        }
    };

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

    return rouille::Response::from_data("application/rss+xml", channel.to_string());
}

fn main() {
    dotenv().ok();
    let cached_posts = Cache::new(get_posts(), std::time::Duration::from_secs(1800)); // cached for 30 min = 1800
    let shared_cache = Arc::new(RwLock::new(cached_posts));

    let index_template = load_template("index").unwrap_or("No template found for /index".to_string());
    let post_template = load_template("posts").unwrap_or("No template found for /posts".to_string());
    let tag_template = load_template("tags").unwrap_or("No template found for /tags".to_string());

    // Preview mode
    let address = format!("0.0.0.0:{}", env::var("PORT").unwrap_or("3123".to_string()));
    println!("Preview server running at {}", address);

    rouille::start_server(&address, move |request| {
        {
            let response = rouille::match_assets(&request, ".");
            // println!("MATCHING {:?}", response);
            if response.is_success() {
                return response;
            }
        }
        router!(request,
            // home page
            (GET) (/) => {
                response_index(shared_cache.clone(), index_template.as_str())
            },
            // content page
            (GET) (/posts/{file_name: String}) => {
                response_view(file_name, post_template.as_str())
            },
            // tag page
            (GET) (/tags/{tag_name: String}) => {
                response_tag(shared_cache.clone(), tag_name, tag_template.as_str())
            },
            // rss feed
            (GET) (/rss) => {
                response_rss(shared_cache.clone())
            },

            _ => rouille::Response::empty_404()
        )
    });
}
