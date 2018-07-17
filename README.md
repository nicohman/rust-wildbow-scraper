# rust-wildbow-scraper

Automatically scrapes wildbow's web serials and compiles them into ebooks

## Available Serials

- Worm
- Pact
- Twig
- Glow-worm
- Ward

## Installation

You'll need [cargo](https://github.com/rust-lang/cargo) installed. Run:

`git clone https://github.com/nicohman/rust-wildbow-scraper.git && cd rust-wildbow-scraper`

`cargo build --release`

`sudo cp targets/release/rust-wildbow-scraper /usr/bin/rust-wildbow-scraper`

## Usage

Run `rust-wildbow-scraper help` to view the list of commands: 

```
Rust Wildbow Scraper v0.0.1
By Nicohman
Commands:
help: Shows this help screen
worm: Scrapes Worm
pact: Scrapes Pact
twig: Scrapes Twig
glow: Scrapes Glow-worm
ward: Scrapes Ward
```

When scraping a book, it'll ask you if you want to include a cover. These are fanart covers and not made or associated with me in any way. The program automatically downloads them from other places and does not have them included.