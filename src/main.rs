extern crate select;
extern crate gen_epub_book;
extern crate reqwest;
use reqwest::Client;
use gen_epub_book::ops::BookElement
use gen_epub_book;
use select::document::Document;
use select::predicate::{Predicate, Name, ;
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
fn get_info(key:&str)
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
        }
    }
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
                        ::std::rocess::exit(64);
        }
    }
    match command.as_ref() {
        "worm" => process_book(download_book(get_info("worm"))),
        "pact" => process_book(download_book(get_info("pact"))),
        "twig" => process_book(download_book(get_info("twig"))),
        "glow" => process_book(download_book(get_info("glow"))),
        "ward" => process_book(download_book(get_info("ward")))
    };
}
fn download_book(book:Book) -> DownloadedBook {
    let done = false;
    let elements = Vec::new(BookElement::Name(book.title), BookElement::Author("John McCrae".to_string()));
    let client = Client::new();
    let done = download_iter(("https://".to_string()+ &book.start, elements, client));
    return DownloadedBook {
        title:book.title,
        content:elements
    }
}
fn download_iter(tup: (String, Vec<BookElement>, Client)) -> (String, Vec<BookElement>, Client)) {
    let page = client.get(tup.0).send().unwrap().text().unwrap();
    let doc = Document::from(page);
    let 
    
}
