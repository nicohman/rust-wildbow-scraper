extern crate structopt;
extern crate directories;
extern crate ego_tree;
extern crate epub_builder;
#[macro_use]
extern crate html5ever;
extern crate html_escape;
extern crate markup5ever;
extern crate reqwest;
extern crate scraper;
extern crate easy_error;
#[macro_use]
extern crate lazy_static;
extern crate xml5ever;

mod cached_client;
mod dom_manipulation;
mod xml_utils;

use cached_client::CachedClient;
use dom_manipulation::{DomOperation, MutableDom};
use html5ever::tree_builder::{NodeOrText, TreeSink};
use structopt::StructOpt;
use directories::ProjectDirs;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ReferenceType, ZipLibrary};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use std::fs::File;
use std::io;
use std::iter::FromIterator;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use easy_error::{ResultExt, Error, err_msg};
use xml_utils::{FilterableTree, html_attr_name, html_elem_name, XmlSerializable};
use std::path::PathBuf;

lazy_static! {
    static ref NEXT_LINK_OVERRIDES: HashMap<String, Url> = HashMap::from([
        ("Last – 20.e6", "https://www.parahumans.net/2020/05/02/last-20-end/"),
    ].map(|(title, url)| (title.to_string(), Url::parse(url).unwrap())));
}

struct Book {
    title: &'static str,
    start: &'static str,
    desc: &'static str,
    date: &'static str,
    cover: Option<&'static str>,
    final_chapter_title: Option<&'static str>,
}

/// scrapes books written by Wildbow like Worm, Ward, Twig ETC and converts it to EPUB format.
#[derive(StructOpt)]
struct Args {
	/// Scrape Worm?
	#[structopt(short, long)]
	worm: bool,
	/// scrape Pact?
	#[structopt(short, long)]
	pact: bool,
	/// scrape Twig?
	#[structopt(short, long)]
	twig: bool,
	/// Scrape Glow Worm?
	#[structopt(short, long)]
	glow_worm: bool,
	/// Scrape Ward?
	#[structopt(short="r", long)]
	ward: bool,
	/// scrape Pale?
	#[structopt(short="l", long)]
	pale: bool,
    #[structopt(short="c", long)]
    claw: bool, 
	/// Scrape them all?
	#[structopt(short, long)]
	all: bool,
    #[structopt(short, long)]
    output: Option<PathBuf>,
	/// get covers? Default is to prompt for each book
	#[structopt(short, long)]
	covers: Option<bool>,
}

struct DownloadedBook {
    title: &'static str,
    builder: EpubBuilder<ZipLibrary>,
}

fn main() -> Result<(), Error> {
    interpret_args()
}

fn get_info(key: &str) -> Option<Book> {
    return Some(match key {
        "worm" => Book {
            title: "Worm",
            start: "https://parahumans.wordpress.com/2011/06/11/1-1/",
            desc: 
                "An introverted teenage girl with an unconventional superpower, Taylor goes out in costume to find escape from a deeply unhappy and frustrated civilian life. Her first attempt at taking down a supervillain sees her mistaken for one, thrusting her into the midst of the local ‘cape’ scene’s politics, unwritten rules, and ambiguous morals. As she risks life and limb, Taylor faces the dilemma of having to do the wrong things for the right reasons.",
            date: "Tue, 19 Nov 2013 00:00:00 +0100",
            cover: Some("https://i.imgur.com/g0fLbQ1.jpg"),
            final_chapter_title: Some("Interlude: End"),
        },
        "pact" => Book {
            title: "Pact",
            start: "https://pactwebserial.wordpress.com/2013/12/17/bonds-1-1/",
            desc: 
                "Blake Thorburn was driven away from home and family by a vicious fight over inheritance, returning only for a deathbed visit with the grandmother who set it in motion. Blake soon finds himself next in line to inherit the property, a trove of dark supernatural knowledge, and the many enemies his grandmother left behind her in the small town of Jacob’s Bell.",
            date: "Sat, 07 Mar 2015 00:00:00 +0100",
            cover: Some("https://preview.redd.it/9scpenoq5v671.png?width=1410&format=png&auto=webp&s=c17e05b90d886ed1858aed33fbeeee37ed35a711"),
            final_chapter_title: Some("Epilogue"),
        },
        "twig" => Book {
            title: "Twig",
            start: "https://twigserial.wordpress.com/2014/12/24/taking-root-1-1/",
            desc: 
                "The year is 1921, and a little over a century has passed since a great mind unraveled the underpinnings of life itself.  Every week, it seems, the papers announce great advances, solving the riddle of immortality, successfully reviving the dead, the cloning of living beings, or blending of two animals into one.  For those on the ground, every week brings new mutterings of work taken by ‘stitched’ men of patchwork flesh that do not need to sleep, or more fearful glances as they have to step off the sidewalks to make room for great laboratory-grown beasts.  Often felt but rarely voiced is the notion that events are already spiraling out of the control of the academies that teach these things. It is only this generation, they say, that the youth and children are able to take the mad changes in stride, accepting it all as a part of day to day life.  Of those children, a small group of strange youths from the Lambsbridge Orphanage stand out, taking a more direct hand in events.",
            date: "Tue, 17 Oct 2017 00:00:00 +0200",
            cover: Some("https://i.imgur.com/3KeIJyz.jpg"),
            final_chapter_title: Some("Forest for the Trees – e.4"),
        },
        "glow" => Book {
            title: "Glow-worm",
            start: "https://parahumans.wordpress.com/2017/10/21/glowworm-p-1/",
            desc: 
                "The bridge between Worm and Ward, Glow-worm introduces readers to the characters of Ward, and the consequences of Gold Morning",
            date: "Sat, 11 Nov 2017 00:00:00 +0100",
            cover: None,
            final_chapter_title: Some("P.9"),
        },
        "ward" => Book {
            title: "Ward",
            start: "https://parahumans.net/2017/09/11/daybreak-1-1/",
            desc: 
                "The unwritten rules that govern the fights and outright wars between ‘capes’ have been amended: everyone gets their second chance.  It’s an uneasy thing to come to terms with when notorious supervillains and even monsters are playing at being hero.  The world ended two years ago, and as humanity straddles the old world and the new, there aren’t records, witnesses, or facilities to answer the villains’ past actions in the present.  One of many compromises, uneasy truces and deceptions that are starting to splinter as humanity rebuilds. None feel the injustice of this new status quo or the lack of established footing more than the past residents of the parahuman asylums.  The facilities hosted parahumans and their victims, but the facilities are ruined or gone; one of many fragile ex-patients is left to find a place in a fractured world.  She’s perhaps the person least suited to have anything to do with this tenuous peace or to stand alongside these false heroes.  She’s put in a position to make the decision: will she compromise to help forge what they call, with dark sentiment, a second golden age?  Or will she stand tall as a gilded dark age dawns?",
            date: "Sat, 11 Nov 2017 00:00:00 +0100",
            cover: Some("https://i.redd.it/2c4czdyhnqv41.jpg"),
            final_chapter_title: Some("Last – 20.end"),
        },
        "pale" => Book {
            title: "Pale",
            start: "https://palewebserial.wordpress.com/2020/05/05/blood-run-cold-0-0/",
            desc: "There are ways of being inducted into the practices, those esoteric traditions that predate computers, cell phones, the engines industry, and even paper and bronze.  Make the right deals, learn the right words to say or symbols to write down, and you can make the wind listen to you, exchange your skin for that of a serpent, or call forth the sorts of monsters that appear in horror movies.",
            date: "Tue, 05 May 2020 00:00:00 +0100",
            cover: Some("https://i.redd.it/xnp5vvxvnr471.png"),
            final_chapter_title: None,
        },
        "claw" => Book {
            title: "Claw",
            start: "https://clawwebserial.blog/2024/03/09/the-point-1-1/",
            desc: "Joshua Munce, Sheila Hardy, Dan Whitely, Max Highland, Tonya Keifer, Marvin Su… this pair has many names, but those names aren’t their own; they’re names to sell.  In a rigged and crumbling system, the only way to get ahead is to circumvent the rules, but that comes with its own risks.  Police, investigations, prison.  There are other ways, more insulated, which are to play assist to help those people.  Helping them to disappear, cleaning up messes, escrow services for the handling of good, payment, or guests.  Always keeping it professional, keeping things insulated, with layers of distance.  When others panic, with too many variables to consider in the heat of the moment, they can do the thinking.  Who would suspect this mom and dad with two kids?",
            date: "Tue, 09 Mar 2024 00:00:00 +0100",
            cover: None,
            final_chapter_title: Some("Loose Ends – E.6"),
        },
        _ => return None,
    });
}

fn prompt_cover(title: &str, url: &str) -> Result<bool, Error> {
    print!(
        "Would you like to include a cover for {}? Cover URL is {}. If it cannot be downloaded, program will not exit gracefully.(y/n)",
        title,
        url
    );
    io::stdout().flush().context("Could not flush stdout")?;
    let reader = io::stdin();
    let mut buf = String::new();
    reader.read_line(&mut buf).context("Could not read line")?;

    Ok(buf.trim() == "y" || buf.trim() == "yes")
}

#[derive(Debug)]
enum MediaType {
    Jpeg,
    Png,
    Svg,
}

impl MediaType {
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }

    pub fn from_url(url: &Url) -> Result<Self, Error> {
        let extension = url.path().split('.').last().ok_or(err_msg(format!("Cannot obtain without suffix specified: {url}")))?;

        if extension == "png" {
            Ok(Self::Png)
        } else if extension == "svg" {
            Ok(Self::Svg)
        } else if extension == "jpg" || extension == "jpeg" {
            Ok(Self::Jpeg)
        } else {
            Err(err_msg(format!("Given URL probably is not one of the supported media types: {url}")))
        }
    }
}

fn interpret_args() -> Result<(), Error> {
    let args = Args::from_args(); // parse command line arguments, print help messages, and make sure all the arguments are valid. This feature is provided by structopt

    let proj_dirs = ProjectDirs::from("net", "Demenses",  "rust-wildbow-scraper");
    let cache_dir = proj_dirs.as_ref().map(|dirs| dirs.cache_dir());
    if let Some(cache_path) = cache_dir {
        println!("Using cache directory: {:?}", cache_path);
    }

    // an anonymous function which adds the book with name name to books if requested is true
    let add_book = |name, requested| {
        if requested {
            process_book(download_book(cache_dir, name, args.covers)?, args.output.clone())?;
        }
        let result: Result<(), Error> = Ok(());
        result
    };
    add_book("worm", args.worm || args.all)?;
    add_book("ward", args.ward || args.all)?;
    add_book("pact", args.pact || args.all)?;
    add_book("pale", args.pale || args.all)?;
    add_book("claw", args.claw || args.all)?;
    add_book("glow", args.glow_worm || args.all)?;
    add_book("twig", args.twig || args.all)
}

fn download_book<P: AsRef<Path>>(
    cache_dir: Option<P>,
    name: &str,
    download_cover_default: Option<bool>
) -> Result<DownloadedBook, Error> {
    let book = get_info(name).ok_or(err_msg(format!("Unknown book {name}")))?;

    let mut builder = EpubBuilder::new(ZipLibrary::new().context("Could not create ZipLibrary")?).context("Could not create EpubBuilder")?;

    let stylesheet = "
        .indent-one {
            margin-left: 2em;
        }
        .indent-two {
            margin-left: 4em;
        }
        .aligncenter, .center {
            text-align: center;
        }
        .right {
            text-align: right;
        }
        .size-full {
            width: 100%;
            height: auto;
            object-fit: contain;
        }
    ";

    builder
    .epub_version(EpubVersion::V30)
    .stylesheet(stylesheet.as_bytes()).context("Could not set stylesheet")?
    .metadata("author", "John McCrae").context("Could not set author metadata")?
    .metadata("title", book.title).context("Could not set title metadata")?
    .metadata("lang", "en-US").context("Could not set language metadata")?
    .metadata("description", book.desc).context("Could not set description metadata")?;
    // date metadata not yet supported
    //.metadata(book.date)?

    let book_cache_dir = cache_dir.map(|dir| dir.as_ref().join(name));
    let client = CachedClient::new(book_cache_dir)?;

    if let Some(cover) = book.cover {
        let download_cover = match download_cover_default {
            Some(download) => download,
            None => prompt_cover(book.title, cover)?
        };
        if download_cover {
            let cover_url = Url::parse(&cover).context(format!("Could not construct url from '{}'", cover))?;
            let res = client.fetch::<Vec<u8>>(&cover_url, false).context(format!("Could not retrieve data from url '{}", cover_url))?;
            if res.is_cached() {
                println!("Using cover from cache for {cover}");
            } else {
                println!("Downloaded cover from {cover}");
            }
            let data = res.contents();
            let filetype = MediaType::from_url(&cover_url)?;
            builder.add_cover_image(format!("cover.{}", filetype.extension()), &**data, filetype.content_type())
                   .context("Could not add cover image")?;
        } else {
            println!("Not using cover.");
        }
    }
    let page_url = Url::parse(&book.start).context(format!("Could not create url from '{}'", book.start))?;
    download_pages(&book, Some(page_url), &mut builder, client)?;

    Ok(DownloadedBook {
        title: book.title,
        builder: builder,
    })
}

fn style_classes(input: ElementRef) -> String {
    let mut attrs: HashMap<&str, &str> = input.value().attrs().collect();

    let mut properties = if let Some(style) = attrs.remove("style") {
        let parsed: Vec<(&str, &str)> = style.split(";")
            .map(|property|
                property
                    .split_once(":")
                    .map(|(name, value)| (name.trim(), value.trim()))
            )
            .filter_map(|property| property)
            .collect();
        HashMap::from_iter(parsed)
    } else {
        HashMap::new()
    };

    let mut classes = Vec::new();

    if let Some(padding_left) = properties.remove("padding-left") {
        if padding_left == "30px" {
            // Indentation in https://twigserial.wordpress.com/2016/12/24/lamb-arc-15/
            classes.push("indent-one");
        } else if padding_left == "40px" {
            // Indentation in https://www.parahumans.net/2019/03/23/heavens-12-9/
            // Indentation in https://palewebserial.wordpress.com/2020/11/21/cutting-class-6-8/
            classes.push("indent-one");
        } else if padding_left == "60px" {
            // Nested indentation in https://twigserial.wordpress.com/2016/11/05/lamb-arc-14/
            classes.push("indent-two");
        } else if padding_left == "80px" {
            // Nested indentation in https://palewebserial.wordpress.com/2022/02/08/gone-and-done-it-17-5/
            classes.push("indent-two");
        } else {
            println!("Warning: Unknown indentation detected: {}", padding_left);
        }
    }

    let align = attrs.remove("align").map(|align| align.to_lowercase());

    if let Some(text_align) = properties.remove("text-align").map(String::from).or(align) {
        if text_align == "center" {
            // Separator ☙ in https://twigserial.wordpress.com/2016/12/17/bitter-pill-15-15/
            // Separator ■ in https://pactwebserial.wordpress.com/category/story/arc-7-void/7-x-histories/
            // Separator 🟂 in https://palewebserial.wordpress.com/2020/05/30/lost-for-words-1-7/
            // Separator ⊙ in https://www.parahumans.net/2019/03/12/heavens-12-f/
            classes.push("center");
        } else if text_align == "right" {
            // Quote attribution in https://pactwebserial.wordpress.com/category/story/arc-7-void/7-x-histories/
            classes.push("right");
        } else if text_align == "left" {
            // Ignore.
        } else {
            println!("Warning: Unknown alignment detected: {}", text_align);
        }
    }

    if !properties.is_empty() {
        println!("Warning: Unhandled properties:");
        for (name, value) in properties {
            println!("  {name}: {value};")
        }
    }

    // dir=ltr is the default and there probably will not be any right-to-left text.
    // https://pactwebserial.wordpress.com/category/story/arc-1-bonds/1-01/
    attrs.remove("dir");

    if !attrs.is_empty() {
        println!("Warning: Unhandled attributes:");
        for (name, value) in attrs {
            println!("  {name}={value:?}")
        }
    }

    if classes.is_empty() {
        "".to_string()
    } else {
        " class=\"".to_string() + &classes.join(" ") + "\""
    }
}

#[test]
fn test_style_classes() {
    let p_selector = Selector::parse("p").unwrap();

    let example = Html::parse_document(r#"<body><p style="text-align:center;">Test</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), r#" class="center""#);

    let example = Html::parse_document(r#"<body><p style="text-align:right;">Test</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), r#" class="right""#);

    let example = Html::parse_document(r#"<body><p style="text-align:left;">Test</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), "");

    // https://parahumans.wordpress.com/category/stories-arcs-1-10/arc-8-extermination/8-x-bonus-interlude/
    let example = Html::parse_document(r#"<body><p style="text-align:left;" align="CENTER">Test</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), r#""#);

    let example = Html::parse_document(r#"<body><p style="padding-left:30px;" align="LEFT">Test.</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), r#" class="indent-one""#);

    let example = Html::parse_document(r#"<body><p align="CENTER">■</p></body>"#);
    assert_eq!(style_classes(example.select(&p_selector).next().unwrap()), r#" class="center""#);
}

lazy_static! {
    static ref META_REFRESH_SELECTOR: Selector = Selector::parse(r#"meta[http-equiv="refresh"]"#).unwrap();
    static ref CONTENT_ELEMENT_SELECTOR: Selector = Selector::parse("div.entry-content p, div.entry-content h1").unwrap();
    static ref LINK_SELECTOR: Selector = Selector::parse("a").unwrap();
    static ref NEXT_LINK_SELECTOR: Selector = Selector::parse(r#"a[rel="next"]"#).unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("title").unwrap();
    static ref IMAGE_SELECTOR: Selector = Selector::parse("img").unwrap();
    static ref CLOUDFLARE_EMAIL_SELECTOR: Selector = Selector::parse("a.__cf_email__[data-cfemail]").unwrap();
    static ref USELESS_SPAN_SELECTOR: Selector = Selector::parse(r#"span:not([class]):not([style]), span.short_text[id="result_box"], span.short_text[id="result_box"] span.hps"#).unwrap();
}

/// Cloudflare mangles anything even vaguely resembling an email into a string that's decoded by
/// javascript on the client. For example, 'Point_Me_@_The_Sky' turns into:
///   '<a href="/cdn-cgi/l/email-protection" class="__cf_email__" data-cfemail="...">[email&nbsp;protected]</a>_The_Sky'
fn fix_cloudflare_links(doc: &mut Html) {
    let cloudflare_links = doc.select(&CLOUDFLARE_EMAIL_SELECTOR).map(|elem| {
        let data = elem.value().attr("data-cfemail").unwrap();
        let bytes = hex::decode(data).expect("mangled email data is not a hex string");
        assert!(bytes.len() >= 4, "mangled email data not long enough");
        let key = bytes[0];
        let decoded = bytes[1..]
            .iter()
            .map(|byte| byte ^ key)
            .collect::<Vec<u8>>();

        DomOperation::ReplaceElement {
            node_id: elem.id(),
            replacement: NodeOrText::AppendText(std::str::from_utf8(&decoded).expect("decoded email isn't a UTF-8 string").into()),
        }
    }).collect();

    doc.perform_operations(cloudflare_links);
}

/// Creates a copy of given image element with cleaned up attributes.
fn clean_up_image(img: &ElementRef, new_src: String) -> Vec<DomOperation<<Html as TreeSink>::Handle>> {
    let img_id = img.id();

    vec![
        // Since we will not download all the differently size versions,
        // let’s ignore srcset & other related attributes.
        // The other sizes are auto-generated by WordPress anyway.
        DomOperation::RemoveAttribute {
            node_id: img_id,
            attr_name: html_attr_name("srcset"),
        },
        DomOperation::RemoveAttribute {
            node_id: img_id,
            attr_name: html_attr_name("sizes"),
        },
        DomOperation::RemoveAttribute {
            node_id: img_id,
            attr_name: html_attr_name("loading"),
        },
        // Replace src.
        DomOperation::SetAttribute {
            node_id: img_id,
            attr_name: html_attr_name("src"),
            value: new_src,
        },
    ]
}

type ImageManager = HashMap<Vec<u8>, (String, &'static str)>;

/// Removes unnecessary attributes on images, changes their sources to point inside the EPUB,
/// and if the paragraph containing the image also contains junk, remove it.
fn clean_up_images(
    doc: &mut Html,
    client: &CachedClient,
    images: &mut ImageManager,
    page_url: &Url,
) -> Result<(), Error> {
    let root = doc.root_element();
    let img_elems = root.select(&IMAGE_SELECTOR);

    let has_single_image = img_elems.clone().count() == 1;
    let should_replace_paragraph_contents = has_single_image && if let Some(link) = root.select(&LINK_SELECTOR).next() {
        // Some Pale chapters (e.g. 18.10) have image paragraphs contaminated with “Next/Previous Chapter” links.
        // If we detect that happened and there is no text in the paragraph outside the link, let’s keep only the image.
        root.text().collect::<String>().trim() == link.text().collect::<String>().trim()
    } else {
        false
    };

    let mut ops = Vec::new();

    for img in img_elems {
        if let Some(src) = img.value().attr("src") {
            let mut image_url = page_url.join(src).context("Could not resolve image URL")?;
            // Remove `w` and `h` parameters from query string to download full-size image.
            image_url.set_query(None);

            let res = client.fetch::<Vec<u8>>(&image_url, false)?;
            if res.is_cached() {
                println!("Found image in cache for {image_url}");
            } else {
                println!("Downloaded image from {image_url}");
            }

            let contents = res.contents().to_vec();

            // Use prefix otherwise epub_builder will produce invalid ids (starting with a number).
            let url_path = format!("images{}", image_url.path());
            let path = if let Some((path, _content_type)) = images.get(&contents) {
                // Pale re-uploads the same header image multiple times.
                // Let’s use the first image if one with the same contents already exists
                // to avoid bloating the EPUB.
                println!("Will re-use {path} instead of {image_url} since it is the same.");

                path.clone()
            } else {
                let content_type = MediaType::from_url(&image_url)?.content_type();
                images.insert(contents, (url_path.clone(), content_type));

                url_path
            };

            let mut new_img_changes = clean_up_image(&img, path);
            ops.append(&mut new_img_changes);
            if should_replace_paragraph_contents {
                // Remove everything and then put just the image back.
                ops.push(
                    DomOperation::RemoveChildren {
                        node_id: root.id(),
                    }
                );
                ops.push(
                    DomOperation::AppendChild {
                        parent_id: root.id(),
                        child_id: img.id(),
                    }
                );
                break;
            }
        }
    }

    doc.perform_operations(ops);

    Ok(())
}

/// Removes spans that are not used for styling the content
/// such as various junk inserted by WordPress’s editor.
fn remove_useless_spans(doc: &mut Html) {
    let useless_spans = doc.select(&USELESS_SPAN_SELECTOR);

    let mut ops = Vec::new();

    for span in useless_spans {
        ops.push(DomOperation::DissolveElement {
            node_id: span.id(),
        })
    }

    doc.perform_operations(ops);
}

#[test]
fn test_useless_span_removal() {
    // Likely created by copying from Translator.
    // https://pactwebserial.wordpress.com/2014/04/03/collateral-4-12/
    let mut example = Html::parse_fragment(r#"<p>“<span id="result_box" class="short_text" lang="ga"><span class="hps">Comhroinn</span> </span><span id="result_box" class="short_text" lang="ga"><span class="hps">liom</span></span><em><span id="result_box" class="short_text" lang="ga">,</span></em>” the woman rasped.</p>"#);
    remove_useless_spans(&mut example);

    assert_eq!(
        Html::parse_fragment("<p>“Comhroinn liom<em>,</em>” the woman rasped.</p>"),
        Html::parse_fragment(&example.root_element().inner_html()),
    );
}

fn download_page(
    client: &CachedClient,
    images: &mut ImageManager,
    page_url: &Url,
    skip_cache: bool,
) -> Result<(String, String, Option<Url>), Error> {
    let res = client.fetch::<String>(page_url, skip_cache)?;
    let is_cached = res.is_cached();
    let page = res.contents();

    let doc = Html::parse_document(page.as_ref());

    // follow redirect if current page uses meta refresh to redirect
    let redirect = doc.select(&META_REFRESH_SELECTOR)
                      .filter_map(|elem| {
                          elem.value().attr("content")
                      })
                      .flat_map(|content| {
                          content.split(';')
                                 .filter(|string| {
                                     string.trim().to_lowercase().starts_with("url=")
                                 })
                      })
                      .next();
    if let Some(redirect_url) = redirect {
        let mut redirect_chars = redirect_url.chars();
        redirect_chars.nth(3); // skip over 'url='
        let page_url = page_url.join(redirect_chars.as_str()).context(format!("Could not resolve url '{}'", redirect_chars.as_str()))?;
        return download_page(client, images, &page_url, skip_cache);
    }

    let next_page = doc.select(&NEXT_LINK_SELECTOR).next();
    let mut title = doc
        .select(&TITLE_SELECTOR)
        .next().ok_or(err_msg("no element named 'title' on page"))?
        .text().collect::<String>()
        .split("|")
        .next().expect("split on string returned no elements")
        .trim()
        .replace(" - Parahumans 2", "")
        .replace(" – Twig", "")
        .replace("Glow-worm – ", "")
        .replace("(Sequel is live!)", "")
        .to_string();
    if &title == "1.01" {
        title = "Bonds 1.1".to_string();
    }
    if is_cached {
        println!("Using {title} from cache for {page_url}");
    } else {
        println!("Downloaded {title} from {page_url}");
    }
    let content_elems = doc
        .select(&CONTENT_ELEMENT_SELECTOR)
        .filter(|elem| {
            // Remove paragraphs which only have text inside links and there is no image.
            // Those are probably “Next/Previous Chapter” links.
            // If there is an image, it is probably due to the link’s paragraph being
            // accidentally smooshed into the paragraph with protagonist image (used e.g. by Pale).
            // We will tackle that during image handling.
            elem.select(&IMAGE_SELECTOR).next().is_some()
            || ! elem.text_filter(|elem| elem.name != html_elem_name("a")).collect::<String>().trim().is_empty()
        });

    let mut body_text = String::new();

    for elem in content_elems {
        let mut doc = Html::parse_fragment(&elem.inner_xml());
        fix_cloudflare_links(&mut doc);
        clean_up_images(
            &mut doc,
            client,
            images,
            page_url,
        )?;
        remove_useless_spans(&mut doc);
        let elem_text = doc.root_element().inner_xml();

        body_text.push_str(&("<p".to_string() + &style_classes(elem) + ">" + &elem_text + "</p>\n"));
    }

    let next_page_url = if let Some(a_element) = next_page {
        Some(page_url.join(a_element.value().attr("href").ok_or(err_msg("<a> link with name 'next' does not have href attribute"))?).context("Could not resolve url")?)
    } else {
        None
    };

    let next_page_url = NEXT_LINK_OVERRIDES.get(&title).cloned().or(next_page_url);

    if next_page_url.is_none() && is_cached {
        // If this was a last chapter and it was cached, let’s try to refetch it
        // in case there is a new chapter link available.
        return download_page(client, images, page_url, true);
    }

    Ok((body_text, title, next_page_url))
}

fn download_pages(
    book: &Book,
    mut link: Option<Url>,
    builder: &mut EpubBuilder<ZipLibrary>,
    client: CachedClient,
) -> Result<(), Error> {

    let mut chapter_number = 1;
    let mut images: ImageManager = HashMap::new();

    while let Some(page_url) = link {
        let (body_text, title, next_page) = download_page(
            &client,
            &mut images,
            &page_url,
            false,
        )?;

        link = next_page;

        if title.starts_with("Glow-worm") && book.title == "Ward" {
            // Glow-worm is also at the beginning of Ward.
            println!("Skipping {title} since it is available separately.");
            continue;
        }

        let escaped_title = html_escape::encode_text(&title);
        let cont = "<?xml version='1.0' encoding='utf-8' ?><html xmlns='http://www.w3.org/1999/xhtml'><head><title>".to_string() + &escaped_title + "</title><meta http-equiv='Content-Type' content ='text/html; charset=utf-8' />\n<link rel='stylesheet' type='text/css' href='stylesheet.css' />\n</head><body><h1>" + &escaped_title + "</h1>\n" + &body_text + "</body></html>";

        // Title here should not need to be escaped but unfortunately,
        // without this the nav.xhtml will contain unescaped &.
        // And we cannot just escape it ourseves or it will be double escaped in toc.ncx.
        // Let’s replace it with small ampersand Unicode character.
        // https://github.com/lise-henry/epub-builder/pull/41
        let title_clean = title.replace("&", "﹠");
        builder.add_content(EpubContent::new(format!("chapter_{}.xhtml", chapter_number), cont.as_bytes()).title(&title_clean).reftype(ReferenceType::Text))
               .context("Could not add chapter")?;

        if Some(title) == book.final_chapter_title.map(|title| title.to_string()) {
            // Stop after the final chapter to avoid including e.g. retrospectives.
            break;
        }

        chapter_number += 1
    }

    for (image, (name, content_type)) in images {
        builder.add_resource(&name, &*image, content_type).context(format!("Could not add image {name}"))?;
    }

    Ok(())
}

fn process_book(mut book: DownloadedBook, output_folder: Option<PathBuf>) -> Result<(), Error> {
    println!("Done downloading {}", book.title);
    let filename = book.title.to_lowercase();
    println!("Converting to epub now at {}.epub", filename);
    let mut zipfile = File::create(output_folder.unwrap_or_default().join(filename+ ".epub")).context("Could not open file")?;
    book.builder.generate(&mut zipfile).context("Could not generate ebook")?;
    println!("Done downloading {}", book.title);
    Ok(())
}
