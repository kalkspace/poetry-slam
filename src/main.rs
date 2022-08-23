#[macro_use]
extern crate rocket;

use std::error::Error;

use flume::{Receiver, Sender};
use printer::PoetryPrinter;
use rocket::{form::Form, tokio::spawn, Build, Rocket, State};
use rocket_dyn_templates::{context, Template};

mod poem_generator;
mod printer;
mod training_data;

#[derive(FromForm)]
struct PoemGenerationForm<'r> {
    training_data: &'r str,
    name: &'r str,
    print_and_hide: bool,
}

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        context! {
            name: "",
            training_data: training_data::DEFAULT_TRAINING_DATA,
        },
    )
}

#[post("/", data = "<poem_generation>")]
async fn generate(
    poem_generation: Form<PoemGenerationForm<'_>>,
    poem_tx: &State<Option<Sender<(String, String)>>>,
) -> Result<Template, String> {
    // hmm. this generates a 200 in case of an error :S
    let poem = poem_generator::generate(poem_generation.training_data)
        .await
        .map_err(|e| e.to_string())?;

    let poem = if poem_generation.print_and_hide {
        if let Some(poem_tx) = poem_tx.inner() {
            poem_tx
                .send((poem_generation.name.to_string(), poem))
                .map_err(|e| e.to_string())?;
        }
        None
    } else {
        Some(poem)
    };

    Ok(Template::render(
        "index",
        context! {
            training_data: poem_generation.training_data,
            name: poem_generation.name,
            poem,
        },
    ))
}

fn rocket(poem_tx: Option<Sender<(String, String)>>) -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, generate])
        .manage(poem_tx)
        .attach(Template::fairing())
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let printer = match PoetryPrinter::new() {
        Ok(mut printer) => {
            let (poem_tx, poem_rx): (Sender<(String, String)>, Receiver<(String, String)>) =
                flume::unbounded();
            let print_task = spawn(async move {
                loop {
                    let (name, training_data) = poem_rx.recv().unwrap();
                    printer.print_poem(&name, &training_data).unwrap();
                }
            });
            Some((poem_tx, print_task))
        }
        Err(e) => {
            eprintln!("Printer init failed: {}. Skipping print.", e);
            None
        }
    };
    if let Some(printer) = printer {
        let _ = rocket(Some(printer.0)).launch().await;
        printer.1.abort();
    } else {
        let _ = rocket(None).launch().await;
    }

    Ok(())
}
