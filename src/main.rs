extern crate select;
extern crate gen_epub_book;
extern crate chrono;
extern crate reqwest;
use reqwest::Client;
use gen_epub_book::ops::{BookElement, EPubBook};
use std::env;
use std::fs::File;
use select::document::Document;
use std::io::stdout;
use select::node::Node;
use chrono::DateTime;
use select::predicate::{Name, And, Class, Descendant};
struct Book {
    title:String,
    start:String
}
struct DownloadedBook {
    title:String,
    content:Vec<BookElement>
}
fn main() {
    interpet_args();
}
fn check_args_num(num: usize, command:&str) -> bool{
    let need = match command {
        _ => 0,
    };
    if num < need {
        false
    } else {
        true
    }
}
fn get_info(key:&str) -> Book{
    return match key {
        "worm" => Book {
            title:String::from("Worm"),
            start:String::from("parahumans.wordpress.com/2011/06/11/1-1/")
        },
        "pact" => Book {
            title:String::from("Pact"),
            start:String::from("pactwebserial.wordpress.com/category/story/arc-1-bonds/1-01/")
        },
        "twig" => Book {
            title:String::from("Twig"),
            start:String::from("twigserial.wordpress.com/2014/12/24/taking-root-1-1/")
        },
        "glow-worm" => Book {
            title:String::from("Glow-worm"),
            start:String::from("parahumans.wordpress.com/2017/10/21/glowworm-p-1/")
        },
        "ward" => Book {
            title:String::from("Ward"),
            start:String::from("parahumans.net/2017/09/11/daybreak-1-1/")
        },
        _ =>  Book {
            title:String::from("Worm"),
            start:String::from("parahumans.wordpress.com/2011/06/11/1-1/")
        },

    };
}
fn interpet_args() {
    let args: Vec<String> = env::args().collect();
    let command : &str;
    if args.len() < 2 {
        command = "help";
    } else {
        command = &args[1];
        if !check_args_num(args.len()- 2, command.as_ref()){
            println!("Not enough arguments for {}", &command);
            ::std::process::exit(64);
        }
    }
    match command.as_ref() {
        "worm" => process_book(download_book(get_info("worm"))),
        "pact" => process_book(download_book(get_info("pact"))),
        "twig" => process_book(download_book(get_info("twig"))),
        "glow" => process_book(download_book(get_info("glow"))),
        "ward" => process_book(download_book(get_info("ward"))),
        _ => process_book(download_book(get_info("worm"))),

    };
}
fn download_book(book:Book) -> DownloadedBook {
    let elements = vec![BookElement::Name(book.title.clone()), BookElement::Author("John McCrae".to_string()), BookElement::Language("en-US".to_string()), BookElement::Date(DateTime::parse_from_rfc3339("2017-02-08T15:30:18+01:00").unwrap())];
    let client = Client::new();
    let done = download_iter(&mut ("https://".to_string()+ &book.start, elements, client));
    return DownloadedBook {
        title:book.title,
        content:done.1
    }
}
fn download_iter( tup: &mut (String, Vec<BookElement>, Client)) -> (String, Vec<BookElement>, Client) {
    let page = tup.2.get(&tup.0).send().unwrap().text().unwrap();
    let doc = Document::from(page.as_ref());
    let check = doc.find(Descendant(And(Name("div"), Class("entry-content")),Descendant(Name("p"),Name("a")))).filter(|x|{
        if x.text().trim() == "Next Chapter" {
            true
        } else {
            false
        }
    }).next();
    let title = doc.find(Name("title")).next().unwrap().text().split("|").next().unwrap().trim().to_string();
    println!("Downloaded {}", title);
    let mut arr = doc.find(Descendant(And(Name("div"), Class("entry-content")),Name("p"))).skip(1).collect::<Vec<Node>>();
    let to_sp = arr.len() -1;
    arr.truncate(to_sp);
    let content = arr.into_iter().fold("<!-- ePub title: \"".to_string()+&title+"\" -->", |acc, x|{
        acc + "<p>"+ &x.text()+"</p>"
    });
    tup.1.push(BookElement::StringContent(content));
    if check.is_none() {
        return tup.clone();
    } else {
        tup.0 = check.unwrap().attr("href").unwrap().to_string();
        return download_iter(tup);
    }
}
fn process_book(book: DownloadedBook) {
    println!("Done downloading {}", book.title);
    let filename = book.title.clone().to_lowercase();
    println!("Converting to epub now at {}.epub", filename);
    let mut processed = EPubBook::from_elements(book.content).unwrap();
    processed.normalise_paths(&["./".parse().unwrap()], false, &mut stdout()).unwrap();
    processed.write_zip(&mut File::create(filename+".epub").unwrap(), false, &mut stdout()).expect("Couldn't export epub");
    println!("Done downloading {}", book.title);
}
