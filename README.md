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

### Build Dependencies

- `rustc`
- `libssl-dev`(On Ubuntu, see [here](https://docs.rs/openssl/latest/openssl/) for other distros)
- `pkg-config`

## Usage

Run `rust-wildbow-scraper --help` to view the list of commands: 

```
USAGE:
    rust-wildbow-scraper [FLAGS] [OPTIONS]

FLAGS:
    -a, --all          Scrape them all?
    -g, --glow-worm    Scrape Glow Worm?
    -h, --help         Prints help information
    -p, --pact         scrape Pact?
    -l, --pale         scrape Pale?
    -t, --twig         scrape Twig?
    -V, --version      Prints version information
    -r, --ward         Scrape Ward?
    -w, --worm         Scrape Worm?

OPTIONS:
    -c, --covers <covers>    get covers? Default is to prompt for each book
```

When scraping a book, it'll ask you if you want to include a cover. These are fanart covers and not made or associated with me in any way. The program automatically downloads them from other places and does not have them included.
