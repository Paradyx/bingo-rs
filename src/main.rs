use build_html::{Container, ContainerType, Html, HtmlContainer, HtmlPage};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{BufRead, BufReader, Write};
use std::iter::repeat;
use std::{fs::File, path::PathBuf};
use std::net::SocketAddr;

use clap::{arg, command, value_parser, ArgGroup, ArgMatches, ArgAction};

use warp::Filter;

fn bingo<'a>(matches: &'a ArgMatches, names: &'a Vec<String> ) -> String {
    let width: usize = *matches
        .get_one("width")
        .expect("argument width is mandatory");
    let height: usize = *matches
        .get_one("height")
        .expect("argument height is mandatory");

    let mut rng = thread_rng();
    let mut shuffled: Vec<String> = names.to_vec();
    shuffled.shuffle(&mut rng);

    let fallback_title = String::from("Bingo");
    let fallback_name = String::from("Joker");
    let default_name = matches
        .get_one::<String>("default")
        .unwrap_or(&fallback_name);
    let default_iter = repeat(default_name);
    let center = matches.get_one::<String>("center");
    let mut bingo = Container::new(ContainerType::Div).with_attributes([("class", "bingo-grid")]);

    let center_index = (height / 2) * width + (width / 2);
    shuffled
        .iter()
        .chain(default_iter)
        .take(width * height)
        .enumerate()
        .map(|(i, name)| match center {
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

h1 {{
    text-align: center;
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
    overflow: hidden;
}}

.center-cell {{ background-color: red; text-align: center;}}
.normal-cell {{ background-color: lightgrey; text-align: center;}}
",
            width, height
        ))
        .with_title(
            matches
                .get_one::<String>("title")
                .unwrap_or(&fallback_title),
        )
        .with_header(
            1,
            matches
                .get_one::<String>("title")
                .unwrap_or(&fallback_title),
        )
        .with_container(bingo)
        .with_container(
            Container::new(ContainerType::Div)
                .with_attributes([("class", "bingo-description")])
                .with_paragraph(
                    matches
                        .get_one::<String>("description")
                        .unwrap_or(&String::from("")),
                ),
        );
    html.to_html_string()
}

#[tokio::main]
async fn service(matches: ArgMatches, names: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = *matches.get_one::<SocketAddr>("bind").unwrap(); 
    let svc = warp::path::end().map(move || warp::reply::html(bingo(&matches, &names)));
    
    warp::serve(svc).run(addr).await;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .propagate_version(true)
        .arg_required_else_help(true)        
        .arg(arg!(-f --file <FILE> "file containing names")
                  .value_parser(value_parser!(PathBuf))
              )
        .arg(arg!(-l --ldap <URL> "url to the ldap OU, example ldap://ds.example.com:389/dc=example,dc=com")
             )
        .arg(arg!(-C --cmd <command> "command to generate names"))
        .group(ArgGroup::new("input")
            .args(["file", "ldap", "cmd"])
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
        .arg(arg!(-c --center <TEXT> "replace center cell with custom text"))
        .arg(arg!(--default <TEXT> "default name, if list runs out of names (default: Joker"))
        .arg(arg!(-t --title <TEXT> "the title (default: Bingo)"))
        .arg(arg!(-d --description <TEXT> "some explanations at the bottom"))
        .arg(arg!(-o --outfile <OUTPUT> "File for the HTML output").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-b --bind <addr> "Create HTTP service and bind to this address").value_parser(value_parser!(SocketAddr)))
        .arg(arg!(--stdout "Print HTML to STDOUT").action(ArgAction::SetTrue))
        .group(ArgGroup::new("mode")
               .args(["outfile", "bind", "stdout"])
               .multiple(false)
               .required(true)
        )
        .get_matches();
    
    
    let names: Vec<String> = if let Some(path) = matches.get_one::<PathBuf>("file") {
        let file = BufReader::new(File::open(path)?);
        file.lines().collect::<std::io::Result<Vec<String>>>()?
    } else if let Some(ldap) = matches.get_one::<String>("ldap") {
        todo!("load from ldap")
    } else if let Some(cmd) = matches.get_one::<String>("cmd") {
        // let buf = if cfg!(target_os = "windows") {
        // Command::new("cmd")
        //         .args(["/C", cmd])
        //         .output()
        //         .expect("failed to execute process")
        // } else {
        // Command::new("sh")
        //         .arg("-c")
        //         .arg(cmd)
        //         .output()
        //         .expect("failed to execute process")
        // };
        // let result: Vec<String> = String::from_utf8(buf.stdout).unwrap().lines();
        // result.lines()
        //file.lines().collect::<std::io::Result<Vec<String>>>()?
        todo!("load from OS command")
    } else {
        unreachable!()
    };
   

    if matches.get_flag("stdout") {
        let html = bingo(&matches, &names);
        std::io::stdout().write_all(&html.as_bytes()).map_err(|err| Box::new(err) as _)    
    } else if let Some(outpath) = matches.get_one::<PathBuf>("outfile") {
        let html = bingo(&matches, &names);
        File::create(outpath)?.write_all(&html.as_bytes()).map_err(|err| Box::new(err) as _)
    } else if let Some(_) = matches.get_one::<SocketAddr>("bind") {
        service(matches, names)
    } else {
        // out.write_all(&html.as_bytes()).map_err(|err| Box::new(err) as _)
        unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`")
    }
}
