use build_html::{Container, ContainerType, Html, HtmlContainer, HtmlPage};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{BufRead, BufReader, Write};
use std::iter::repeat;
use std::{fs::File, path::PathBuf};

use clap::{arg, command, value_parser, ArgGroup};

fn main() -> std::io::Result<()> {
    let matches = command!()
        .arg(arg!(-f --file <FILE> "file containing names")
                  .value_parser(value_parser!(PathBuf))
              )
        .arg(arg!(-l --ldap <URL> "url to the ldap OU, example ldap://ds.example.com:389/dc=example,dc=com")
             )
        .group(ArgGroup::new("input")
            .args(["file", "ldap"])
            .multiple(false)
            .required(true))
        .arg(arg!(-x --width <WIDTH> "width of the grid")
             .value_parser(value_parser!(usize))
             .required(true)
             )
        .arg(arg!(-y --height <HEIGHT> "height of the grid")
             .value_parser(value_parser!(usize))
             .required(true)
             )
        .arg(arg!(-o --output <OUTPUT> "file for the HTML output (default: STDOUT)")
             .value_parser(value_parser!(PathBuf))
             )
        .arg(arg!(-c --center <TEXT> "replace center cell with custom text"))
        .arg(arg!(-d --default <TEXT> "default name, if list runs out of names"))
        .get_matches();

    let mut names: Vec<String> = if let Some(path) = matches.get_one::<PathBuf>("file") {
        let file = BufReader::new(File::open(path)?);
        file.lines().collect::<std::io::Result<Vec<String>>>()?
    } else if let Some(ldap) = matches.get_one::<String>("ldap") {
        todo!("load from ldap")
    } else {
        unreachable!()
    };

    let width: usize = *matches
        .get_one("width")
        .expect("argument width is mandatory");
    let height: usize = *matches
        .get_one("height")
        .expect("argument height is mandatory");

    let mut rng = thread_rng();
    names.as_mut_slice().shuffle(&mut rng);

    let fallback_name = String::from("Joker");
    let default_name = matches.get_one::<String>("default").unwrap_or(&fallback_name);
    let default_iter = repeat(default_name);

    let mut bingo = Container::new(ContainerType::Div).with_attributes([("class", "bingo-grid")]);

    let center_index = (height / 2) * width + (width / 2);
    names
        .iter()
        .chain(default_iter)
        .take(width * height)
        .enumerate()
        .map(|(i, name)| match matches.get_one::<String>("center") {
            Some(center_text) if i == center_index => Container::new(ContainerType::Div)
                .with_attributes([("class", "bingo-cell center-cell")])
                .with_paragraph(center_text),
            _ => Container::new(ContainerType::Div)
                .with_attributes([("class", "bingo-cell normal-cell")])
                .with_paragraph(name),
        })
        .for_each(|cell| bingo.add_container(cell));

    let html = HtmlPage::new()
        .with_style(format!(
            // escaping of '{' and '}' is '{{' and '}}'.
            "
* {{
    box-sizing: border-box;
    padding: 0;
    margin: 0;
}}

.bingo-grid {{ 
    margin: 1em;
    display: grid;
    grid-template-columns: repeat({}, minmax(100px, 1fr));
    grid-template-rows: repeat({}, minmax(100px, 1fr));
    grid-gap: 1em;
}}

.bingo-grid div {{ 
    aspect-ratio: 1; 

    display: flex;
    align-items: center;
    justify-content: center;
}}

.center-cell {{ background-color: red; }}
.normal-cell {{ background-color: lightgrey; }}
",
            width, height
        ))
        .with_container(bingo);
    
    let mut out: Box<dyn Write> = if let Some(outpath) = matches.get_one::<PathBuf>("output") {
        Box::new(File::create(outpath)?)
    } else {
        Box::new(std::io::stdout())
    };
    out.write_all(&html.to_html_string().as_bytes())
}
