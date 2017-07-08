#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(alloc_system)]
extern crate alloc_system;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate rocket;
extern crate rocket_contrib;
extern crate ordered_float;

use rocket_contrib::JSON;
use ordered_float::{NotNaN, FloatIsNaN};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    id: String,
    duration: u16,
    priority: u8,
}

#[derive(Serialize, Deserialize)]
struct Placement {
    id: String,
    start: u32,
    end: u32,
}

#[post("/", data = "<tasks>")]
fn schedule(tasks: JSON<Vec<Task>>) -> Result<JSON<Vec<Placement>>, Box<Error>> {
    let tasks = tasks.into_inner();
    let mut placements = Vec::with_capacity(tasks.capacity());
    let mut tasks: Vec<_> = tasks.iter().map(|x| Some(x)).collect();
    let mut now = 0;
    loop {
        let task_utils: Result<Vec<_>, _> = tasks
            .iter()
            .map(|t| t.map_or(Ok(NotNaN::from(0.0)), |t| utility(t)))
            .enumerate()
            .map(|ui| {
                let (u, i) = ui;
                i.map(|i| (u, i))
            })
            .collect();
        let task_utils = task_utils?;
        let t = task_utils
            .into_iter()
            .max_by_key(|x| x.1)
            .and_then(|t| tasks.swap_remove(t.0));
        if let Some(t) = t {
            let after = now + t.duration as u32;
            placements.push(Placement {
                id: t.id.clone(),
                start: now,
                end: after,
            });
            now = after + 1;
        } else {
            break;
        };
    }
    Ok(JSON(placements))
}


fn utility(t: &Task) -> Result<NotNaN<f64>, FloatIsNaN> {
    NotNaN::new(t.priority as f64 / t.duration as f64)
}

#[get("/healthz")]
fn health_check() {}


fn main() {
    rocket::ignite()
        .mount("/", routes![schedule, health_check])
        .launch();
}
