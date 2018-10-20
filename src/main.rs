extern crate select;
extern crate gen_epub_book;
extern crate url;
extern crate chrono;
extern crate reqwest;
extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, PushConst, StoreConst};
use reqwest::Client;
use url::Url;
use std::path::PathBuf;
use gen_epub_book::ops::{BookElement, EPubBook};
use std::env;
use std::io;
use std::io::Write;
use std::fs;
use std::fs::File;
use select::document::Document;
use std::io::stdout;
use select::node::Node;
use chrono::DateTime;
use std::fs::OpenOptions;
use select::predicate::{Name, And, Class, Descendant};
use std::thread::Builder;
const FILE_USE: bool = true;
const BOOKS: [&str; 5] = ["worm", "pact", "twig", "glow", "ward"];
struct Book {
    title: String,
    start: String,
    desc: String,
    date: String,
    cover: Option<String>,
}

struct DownloadedBook {
    title: String,
    content: Vec<BookElement>,
}
fn main() {
    let builder = Builder::new().name("reductor".into()).stack_size(
        32 * 1024 * 1024,
    ); //32 MB of stack space
    let handler = builder.spawn(|| { interpet_args(); }).unwrap();
    handler.join().unwrap();
}
fn get_info(key: &str) -> Book {
    return match key {
        "worm" => Book {
            title: String::from("Worm"),
            start: String::from("parahumans.wordpress.com/2011/06/11/1-1/"),
            desc: String::from(
                "An introverted teenage girl with an unconventional superpower, Taylor goes out in costume to find escape from a deeply unhappy and frustrated civilian life. Her first attempt at taking down a supervillain sees her mistaken for one, thrusting her into the midst of the local ‘cape’ scene’s politics, unwritten rules, and ambiguous morals. As she risks life and limb, Taylor faces the dilemma of having to do the wrong things for the right reasons.",
            ),
            date: String::from("Tue, 19 Nov 2013 00:00:00 +0100"),
            cover: Some("https://i.imgur.com/g0fLbQ1.jpg".to_string()),
        },
        "pact" => Book {
            title: String::from("Pact"),
            start: String::from(
                "pactwebserial.wordpress.com/category/story/arc-1-bonds/1-01/",
            ),
            desc: String::from(
                "Blake Thorburn was driven away from home and family by a vicious fight over inheritance, returning only for a deathbed visit with the grandmother who set it in motion. Blake soon finds himself next in line to inherit the property, a trove of dark supernatural knowledge, and the many enemies his grandmother left behind her in the small town of Jacob’s Bell.",
            ),
            date: String::from("Sat, 07 Mar 2015 00:00:00 +0100"),
            cover: Some("https://i.redd.it/uyfiofnoko8z.png".to_string()),
        },
        "twig" => Book {
            title: String::from("Twig"),
            start: String::from("twigserial.wordpress.com/2014/12/24/taking-root-1-1/"),
            desc: String::from(
                "The year is 1921, and a little over a century has passed since a great mind unraveled the underpinnings of life itself.  Every week, it seems, the papers announce great advances, solving the riddle of immortality, successfully reviving the dead, the cloning of living beings, or blending of two animals into one.  For those on the ground, every week brings new mutterings of work taken by ‘stitched’ men of patchwork flesh that do not need to sleep, or more fearful glances as they have to step off the sidewalks to make room for great laboratory-grown beasts.  Often felt but rarely voiced is the notion that events are already spiraling out of the control of the academies that teach these things. It is only this generation, they say, that the youth and children are able to take the mad changes in stride, accepting it all as a part of day to day life.  Of those children, a small group of strange youths from the Lambsbridge Orphanage stand out, taking a more direct hand in events.",
            ),
            date: String::from("Tue, 17 Oct 2017 00:00:00 +0200"),
            cover: Some("https://i.imgur.com/3KeIJyz.jpg".to_string()),
        },
        "glow" => Book {
            title: String::from("Glow-worm"),
            start: String::from("parahumans.wordpress.com/2017/10/21/glowworm-p-1/"),
            desc: String::from(
                "The bridge between Worm and Ward, Glow-worm introduces readers to the characters of Ward, and the consequences of Gold Morning",
            ),
            date: String::from("Sat, 11 Nov 2017 00:00:00 +0100"),
            cover: None,
        },
        "ward" => Book {
            title: String::from("Ward"),
            start: String::from("parahumans.net/2017/09/11/daybreak-1-1/"),
            desc: String::from(
                "The unwritten rules that govern the fights and outright wars between ‘capes’ have been amended: everyone gets their second chance.  It’s an uneasy thing to come to terms with when notorious supervillains and even monsters are playing at being hero.  The world ended two years ago, and as humanity straddles the old world and the new, there aren’t records, witnesses, or facilities to answer the villains’ past actions in the present.  One of many compromises, uneasy truces and deceptions that are starting to splinter as humanity rebuilds. None feel the injustice of this new status quo or the lack of established footing more than the past residents of the parahuman asylums.  The facilities hosted parahumans and their victims, but the facilities are ruined or gone; one of many fragile ex-patients is left to find a place in a fractured world.  She’s perhaps the person least suited to have anything to do with this tenuous peace or to stand alongside these false heroes.  She’s put in a position to make the decision: will she compromise to help forge what they call, with dark sentiment, a second golden age?  Or will she stand tall as a gilded dark age dawns?",
            ),
            date: String::from("Sat, 11 Nov 2017 00:00:00 +0100"),
            cover: None,
        },
        _ => Book {
            title: String::from("Worm"),
            start: String::from("parahumans.wordpress.com/2011/06/11/1-1/"),
            desc: String::from(
                "An introverted teenage girl with an unconventional superpower, Taylor goes out in costume to find escape from a deeply unhappy and frustrated civilian life. Her first attempt at taking down a supervillain sees her mistaken for one, thrusting her into the midst of the local ‘cape’ scene’s politics, unwritten rules, and ambiguous morals. As she risks life and limb, Taylor faces the dilemma of having to do the wrong things for the right reasons.",
            ),
            date: String::from("Tue, 19 Nov 2013 00:00:00 +0100"),
            cover: Some("https://i.imgur.com/g0fLbQ1.jpg".to_string()),
        },

    };
}
fn prompt_cover(title: String, url: String) -> bool {
    print!(
        "Would you like to include a cover for {}? Cover URL is {}. If it cannot be downloaded, program will not exit gracefully.(y/n)",
        title,
        url
    );
    io::stdout().flush().ok().expect("Could not flush stdout");
    let reader = io::stdin();
    let mut buf = String::new();
    (reader).read_line(&mut buf).unwrap();
    buf == "y".to_string() || buf == "yes".to_string()
}
fn interpet_args() {

    let mut which: Vec<String> = Vec::new();
    let mut yes = "";
    {

        let mut parser = ArgumentParser::new();
        parser.set_description("Scrapes Wildbow's web serials");
        parser
            .refer(&mut which)
            .add_option(
                &["-w", "--worm"],
                PushConst("worm".to_string()),
                "Scrape Worm",
            )
            .add_option(
                &["-p", "--pact"],
                PushConst("pact".to_string()),
                "Scrape Pact",
            )
            .add_option(
                &["-t", "--twig"],
                PushConst("twig".to_string()),
                "Scrape Twig",
            )
            .add_option(
                &["-g", "--glow"],
                PushConst("glow".to_string()),
                "Scrape Glow-worm",
            )
            .add_option(
                &["-r", "--ward"],
                PushConst("ward".to_string()),
                "Scrape Ward",
            )
            .add_option(&["-a", "--all"], PushConst("all".to_string()), "Scrape all");
        parser
            .refer(&mut yes)
            .add_option(
                &["-y", "--yes"],
                StoreConst("yes"),
                "Preemptively download all covers",
            )
            .add_option(
                &["-n", "--no"],
                StoreConst("no"),
                "Preemptively decline all covers",
            );
        parser.parse_args_or_exit();
    }
    let mut r: Option<bool>;
    if yes == "yes" {
        r = Some(true);
    } else if yes == "no" {
        r = Some(false);
    } else {
        r = None;
    }
    if which
        .iter()
        .position(|ref s| s == &&"all".to_string())
        .is_some()
    {
        gen_all(r);
    } else {
        for b in which {
            process_book(download_book(get_info(&b), r));
        }
    }

}
fn gen_all(yes: Option<bool>) {
    for book in BOOKS.iter() {
        let info = get_info(book);
        println!("Now downloading {}", info.title);
        process_book(download_book(info, yes));
    }
}
fn get_con_dir() -> String {
    if fs::metadata("/tmp").is_ok() {
        String::from("/tmp/content")
    } else {
        String::from("content")
    }
}
fn download_book(book: Book, yes: Option<bool>) -> DownloadedBook {
    let mut elements = vec![
        BookElement::Name(book.title.clone()),
        BookElement::Author("John McCrae".to_string()),
        BookElement::Language("en-US".to_string()),
        BookElement::Date(DateTime::parse_from_rfc2822(&book.date).unwrap()),
        BookElement::StringDescription(book.desc),
    ];
    if book.cover.is_some() {
        let cover = book.cover.unwrap();
        if yes.is_none() {
            if prompt_cover(book.title.clone(), cover.clone()) {
                elements.push(BookElement::NetworkCover(Url::parse(&cover).unwrap()));
            }
        } else if yes.is_some() {
            if yes.unwrap() {
                elements.push(BookElement::NetworkCover(Url::parse(&cover).unwrap()));
            }
        }

    }
    let client = Client::new();
    if FILE_USE {
        if !fs::metadata(get_con_dir()).is_err() {
            print!(
                "Content directory is already there. Would you like to remove {}?",
                get_con_dir()
            );
            io::stdout().flush().ok().expect("Could not flush stdout");
            let reader = io::stdin();
            let mut buf = String::new();
            (reader).read_line(&mut buf).unwrap();
            buf = buf.trim().to_string();
            if buf == "y".to_string() || buf == "yes".to_string() {
                println!("Removed {}", get_con_dir());
                fs::remove_dir_all(get_con_dir()).unwrap();
                fs::create_dir(get_con_dir()).unwrap();
            } else {
                println!("Exiting");
                ::std::process::exit(73);
            }
        } else {
            fs::create_dir(get_con_dir()).unwrap();
        }
    }
    let done = download_iter(&mut (
        "https://".to_string() + &book.start,
        elements,
        client,
    ));
    return DownloadedBook {
        title: book.title,
        content: done.1,
    };
}
fn download_iter(
    tup: &mut (String, Vec<BookElement>, Client),
) -> (String, Vec<BookElement>, Client) {
    let page = tup.2.get(&tup.0).send().unwrap().text().unwrap();
    let doc = Document::from(page.as_ref());
    let check = doc.find(Descendant(
        And(Name("div"), Class("entry-content")),
        Descendant(Name("p"), Name("a")),
    )).filter(|x| if x.text().trim() == "Next Chapter" ||
            x.text().trim() == "Next"
        {
            true
        } else {
            false
        })
        .next();
    let mut title = doc.find(Name("title"))
        .next()
        .unwrap()
        .text()
        .split("|")
        .next()
        .unwrap()
        .trim()
        .replace(" - Parahumans 2", "")
        .replace(" – Twig", "")
        .replace("Glow-worm – ", "")
        .replace("(Sequel is live!)", "")
        .to_string();
    if &title == "1.01" {
        title = "Bonds 1.1".to_string();
    }
    println!("Downloaded {}", title);
    let mut arr = doc.find(Descendant(
        And(Name("div"), Class("entry-content")),
        Name("p"),
    )).skip(1)
        .collect::<Vec<Node>>();
    let to_sp = arr.len() - 1;
    arr.truncate(to_sp);
    let num = tup.1.len().clone().to_string();
    let cont = arr.into_iter().fold("<?xml version='1.0' encoding='utf-8' ?><html xmlns='http://www.w3.org/1999/xhtml'><head><title>".to_string()+&title+"</title><meta http-equiv='Content-Type' content ='text/html'></meta><!-- ePub title: \"" +&title+ "\" -->\n</head><body><h1>"+&title+"</h1>\n", |acc, x|{
        acc + "<p>"+ &x.inner_html().replace("&nbsp;","&#160;").replace("<br>","<br></br>").replace("& ", "&amp;").replace("<Walk or->","&lt;Walk or-&gt;").replace("<Walk!>","&lt;Walk!&gt;")+"</p>\n"
    })+"</body></html>";
    if FILE_USE {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(get_con_dir() + "/" + &num + ".html")
            .unwrap();
        file.write_all((cont).as_bytes()).unwrap();
        tup.1.push(BookElement::Content(
            PathBuf::from(get_con_dir() + "/" + &num + ".html"),
        ));
    } else {
        tup.1.push(BookElement::StringContent(cont));
    }
    if check.is_none() || title == "P.9" {
        return tup.clone();
    } else {
        tup.0 = check.unwrap().attr("href").unwrap().to_string();
        if !tup.0.contains("https") {
            tup.0 = "https:".to_string() + &tup.0;
        }
        return download_iter(tup);
    }
}
fn process_book(book: DownloadedBook) {
    println!("Done downloading {}", book.title);
    let filename = book.title.clone().to_lowercase();
    println!("Converting to epub now at {}.epub", filename);
    let mut processed = EPubBook::from_elements(book.content).unwrap();
    processed
        .normalise_paths(&["./".parse().unwrap()], false, &mut stdout())
        .unwrap();
    processed
        .write_zip(
            &mut File::create(filename + ".epub").unwrap(),
            false,
            &mut stdout(),
        )
        .expect("Couldn't export epub");
    if FILE_USE {
        fs::remove_dir_all(get_con_dir()).unwrap();
    }
    println!("Done downloading {}", book.title);
}
