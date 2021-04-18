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

` cargo install --path .`

## Usage

Run `rust-wildbow-scraper --help` to view the list of commands: 

```
Usage:
   rust-wildbow-scraper [OPTIONS]

Scrapes wildbow's web serials

Optional arguments:
  -h,--help             Show this help message and exit
  -w,--worm             Scrape Worm
  -p,--pact             Scrape Pact
  -t,--twig             Scrape Twig
  -g,--glow             Scrape Glow-worm
  -r,--ward             Scrape Ward
  -a,--pale             Scrape Pale
  -a,--all              Scrape all
  -y,--yes              Preemptively download all covers
  -n,--no               Preemptively decline all covers
```

When scraping a book, it'll ask you if you want to include a cover. These are fanart covers and not made or associated with me in any way. The program automatically downloads them from other places and does not have them included.
