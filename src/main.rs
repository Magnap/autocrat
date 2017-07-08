#![feature(plugin)]
#![plugin(rocket_codegen)]

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

type Timestamp = u32;

#[derive(Serialize, Deserialize, Clone)]
struct Placement<T> {
    start: Timestamp,
    end: Timestamp,
    value: T,
}

// TODO
type Constraint = ();

type Utility = NotNaN<f64>;

#[post("/", data = "<tasks>")]
fn scheduler(tasks: JSON<Vec<Task>>) -> Result<JSON<Vec<Placement<String>>>, Box<Error>> {
    let tasks = tasks.into_inner();
    let placements = schedule(&tasks, &Vec::new(), 0)?;
    let placements = placements
        .into_iter()
        .map(|p| {
            Placement {
                start: p.start,
                end: p.end,
                value: p.value.id.clone(),
            }
        })
        .collect();
    Ok(JSON(placements))
}

#[inline]
fn schedule<'a>(
    tasks: &'a [Task],
    constraints: &[Constraint],
    start: Timestamp,
) -> Result<Vec<Placement<&'a Task>>, Box<Error>> {
    let mut placements: Vec<Placement<&Task>> = Vec::with_capacity(tasks.len());
    let mut tasks: Vec<_> = tasks.iter().map(|x| Some(x)).collect();
    loop {
        let mut potentials = Vec::with_capacity(tasks.len());
        for (i, t) in tasks.iter().enumerate() {
            if let Some(t) = *t {
                let now = placements.last().map(|x| x.end);
                let now = match now {
                    Some(now) => now + 1,
                    None => start,
                };
                let new = Placement {
                    start: now,
                    end: now + t.duration as u32,
                    value: t,
                };
                let mut potential = placements.clone();
                potential.push(new.clone());
                potentials.push((evaluate(&potential, &constraints)?, i, new));
            }
        }
        let p = potentials.into_iter().max_by_key(|x| x.0);
        if let Some((_, i, p)) = p {
            tasks[i] = None;
            placements.push(p);
        } else {
            break;
        }
    }
    Ok(placements)
}

fn evaluate(
    schedule: &[Placement<&Task>],
    constraints: &[Constraint],
) -> Result<Utility, FloatIsNaN> {
    let utils = schedule.iter().map(|p| {
        let t = p.value;
        t.priority as f64 / t.duration as f64
    });
    let sum = utils.into_iter().sum();
    let sum = NotNaN::new(sum)?;
    Ok(sum)
}

#[get("/healthz")]
fn health_check() {}


fn main() {
    rocket::ignite()
        .mount("/", routes![scheduler, health_check])
        .launch();
}
