use std::collections::HashMap;
use scraper::{Html, Selector};
use chrono::{Timelike, Local};

/*
-Load data into a Vec of rooms, which includes name, size, and Vec of reservations
-Function to get free rooms given time, duration, and size
Accept command line args for start and duration
Add "watch" flag to run continuously and watch for a room to free up
Move Reservation struct and logic to separate file?
*/

// 58px = 1 hour
// 60 / 58 = 1.03448275862 (minutes per px)
// 60 / 58 * 60 = 62.0689655172 (seconds per px)
const SECONDS_PER_WIDTH_PX: f64 = 62.0689655172;

#[derive(Debug)]
struct Room {
    name: String,
    floor: i32,
    size: u32,
    reservations: Vec<Reservation>,
}

#[derive(Debug)]
struct Reservation {
    start: u32,
    end: u32,
}

fn get_free_rooms(rooms: HashMap<String, Room>, start: u32, end: u32) {
    for (_, room) in rooms.iter() {
        let mut free = true;
        for reservation in &room.reservations {
            if (start > reservation.start && start < reservation.end) // Starts during this reservation
                || (end > reservation.start && end < reservation.end) // Ends during this reservation
                || (start < reservation.start && end > reservation.end) // This reservation is in the middle of our target
            {
                // Starts during this reservation
                free = false;
                break;
            }
        }
        if free {
            println!("{} (seats {})", room.name, room.size);
        }
    }
}

fn main() -> Result<(), reqwest::Error> {
    let mut rooms = HashMap::new();

    let html = reqwest::get("https://industryrinostation.roomzilla.net/")?
        .text()?;

    let document = Html::parse_document(&html);

    let table_selector = Selector::parse("table#timeline tbody tr").unwrap();
    let name_selector = Selector::parse("td.name").unwrap();
    let floor_selector = Selector::parse("td.floor").unwrap();
    let size_selector = Selector::parse("td.size").unwrap();
    for element in document.select(&table_selector) {
        let name_element = element.select(&name_selector).next().unwrap();
        let floor_element = element.select(&floor_selector).next().unwrap();
        let size_element = element.select(&size_selector).next().unwrap();
        rooms.insert(name_element.value().attr("data-sort").unwrap().to_owned(), Room {
            name: name_element.value().attr("data-sort").unwrap().to_owned(),
            floor: floor_element.value().attr("data-sort").unwrap().parse::<i32>().unwrap(),
            size: size_element.value().attr("data-sort").unwrap().parse::<u32>().unwrap(),
            reservations: vec![],
        });
    }

    let reserved_selector = Selector::parse("div.reserved").unwrap();
    for element in document.select(&reserved_selector) {
        // let day = element.value().attr("day").unwrap();
        let room_name = element.value().attr("room_name").unwrap();
        let start = element.value().attr("seconds").unwrap().parse::<f64>().unwrap();
        let style = element.value().attr("style").unwrap();
        let duration = (&style[7..style.find("px;").unwrap()].parse::<f64>().unwrap() * SECONDS_PER_WIDTH_PX).round();
        if let Some(room) = rooms.get_mut(room_name) {
            room.reservations.push(Reservation {
                start: start as u32,
                end: (start + duration) as u32,
            });
        }
    }

    // println!("{:?}", rooms);

    let now = Local::now();
    // Get free rooms for right now, available for 60 minutes
    println!("Finding rooms available for 60 minutes right now");
    get_free_rooms(rooms, now.num_seconds_from_midnight(), now.num_seconds_from_midnight() + (60 * 60));

    Ok(())
}
