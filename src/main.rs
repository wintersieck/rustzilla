use std::collections::HashMap;
use std::error::Error;
use scraper::{Html, Selector};
use chrono::{Timelike, Local, DateTime, Duration};
use clap::{Arg, App};

// Roomzilla doesn't give us an end time or duration, so we have to infer it by the width of the reservation element
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

fn parse_time_arg(arg_value: Option<&str>, default: DateTime<Local>) -> Result<DateTime<Local>, Box<dyn Error>> {
    match arg_value {
        Some(start) => {
            let split = start.split(":").collect::<Vec<_>>();
            Ok(Local::now()
                .with_hour(split.get(0).ok_or("Could not parse hour")?.parse::<u32>()?).ok_or("Could not parse hour")?
                .with_minute(split.get(1).ok_or("Could not parse minute")?.parse::<u32>()?).ok_or("Could not parse minute")?
            )
        },
        None => Ok(default)
    }
}

fn scrape_rooms() -> Result<HashMap<String, Room>, Box<dyn Error>> {
    let mut rooms = HashMap::new();

    let html = reqwest::get("https://industryrinostation.roomzilla.net/")?
        .text()?;

    let document = Html::parse_document(&html);

    let table_selector = Selector::parse("table#timeline tbody tr").expect("Failed to parse selector");
    let name_selector = Selector::parse("td.name").expect("Failed to parse selector");
    let floor_selector = Selector::parse("td.floor").expect("Failed to parse selector");
    let size_selector = Selector::parse("td.size").expect("Failed to parse selector");
    for element in document.select(&table_selector) {
        let name_element = element.select(&name_selector).next().ok_or("Failed to find name element")?;
        let floor_element = element.select(&floor_selector).next().ok_or("Failed to find floor element")?;
        let size_element = element.select(&size_selector).next().ok_or("Failed to find size element")?;
        rooms.insert(name_element.value().attr("data-sort").ok_or("Failed to parse name from name element")?.to_owned(), Room {
            name: name_element.value().attr("data-sort").ok_or("Failed to parse name from name element")?.to_owned(),
            floor: floor_element.value().attr("data-sort").ok_or("Failed to parse floor from floor element")?.parse::<i32>()?,
            size: size_element.value().attr("data-sort").ok_or("Failed to parse size from size element")?.parse::<u32>()?,
            reservations: vec![],
        });
    }

    let reserved_selector = Selector::parse("div.reserved").expect("Failed to parse selector");
    for element in document.select(&reserved_selector) {
        let room_name = element.value().attr("room_name").ok_or("Failed to parse room name for reservation")?;
        let start = element.value().attr("seconds").ok_or("Failed to parse room seconds for reservation start")?.parse::<f64>()?;
        let style = element.value().attr("style").ok_or("Failed to parse room style for reservation duration")?;
        let duration = (&style[7..style.find("px;").ok_or("Failed to parse duration from room style")?].parse::<f64>()? * SECONDS_PER_WIDTH_PX).round();
        if let Some(room) = rooms.get_mut(room_name) {
            room.reservations.push(Reservation {
                start: start as u32,
                end: (start + duration) as u32,
            });
        }
    }
    Ok(rooms)
}

fn print_free_rooms(rooms: HashMap<String, Room>, start: DateTime<Local>, end: DateTime<Local>) {
    println!("Free rooms available from {} to {}", start.format("%I:%M%P").to_string(), end.format("%I:%M%P").to_string());
    for (_, room) in rooms.iter() {
        let mut free = true;
        for reservation in &room.reservations {
            if (start.num_seconds_from_midnight() > reservation.start && start.num_seconds_from_midnight() < reservation.end) // Starts during this reservation
                || (end.num_seconds_from_midnight() > reservation.start && end.num_seconds_from_midnight() < reservation.end) // Ends during this reservation
                || (start.num_seconds_from_midnight() < reservation.start && end.num_seconds_from_midnight() > reservation.end) // This reservation is in the middle of our target
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

fn main() {
    let cli_args = App::new("Rustzilla")
                          .version("1.0")
                          .author("Allan Wintersieck <awintersieck@gmail.com>")
                          .about("Scrapes Industry RiNo Roomzilla")
                          .arg(Arg::with_name("start")
                               .short("s")
                               .long("start")
                               .value_name("START")
                               .help("Start time in format of HH:MM (24-hour clock)")
                               .takes_value(true))
                          .arg(Arg::with_name("end")
                               .short("e")
                               .long("end")
                               .value_name("END")
                               .help("End time in format of HH:MM (24-hour clock)")
                               .takes_value(true))
                          .get_matches();

    let start = match parse_time_arg(cli_args.value_of("start"), Local::now()) {
        Ok(start) => start,
        Err(err) => panic!("Failed to parse start time argument: {}", err)
    };
    let end = match parse_time_arg(cli_args.value_of("end"), start + Duration::seconds(3600)) {
        Ok(end) => end,
        Err(err) => panic!("Failed to parse end time argument: {}", err)
    };

    let rooms = match scrape_rooms() {
        Ok(rooms) => rooms,
        Err(err) => panic!("Failed to scrape room and reservation data: {}", err)
    };

    print_free_rooms(rooms, start, end);
}
