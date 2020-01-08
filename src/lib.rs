use std::collections::HashMap;
use std::error::Error;
use scraper::{Html, Selector};
use chrono::{Timelike, Local, DateTime, Duration};
use clap::ArgMatches;

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
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap()
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

pub fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let start = parse_time_arg(args.value_of("start"), Local::now())?;

    let end = parse_time_arg(args.value_of("end"), start + Duration::seconds(3600))?;

    let rooms = scrape_rooms()?;

    print_free_rooms(rooms, start, end);

    Ok(())
}

#[cfg(test)]
mod test_parse_time_arg {
    use super::*;

    fn make_local_time(hour: u32, minute: u32) -> DateTime<Local> {
        Local::now()
            .with_hour(hour).unwrap()
            .with_minute(minute).unwrap()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap()
    }

    #[test]
    fn parses_simple_time() {
        let time = "8:30";
        let default = make_local_time(12, 0);
        let expected = make_local_time(8, 30);
        
        assert_eq!(
            parse_time_arg(Some(time), default).unwrap(),
            expected
        );
    }

    #[test]
    fn parses_time_with_zero_padding() {
        let time = "08:05";
        let default = make_local_time(12, 0);
        let expected = make_local_time(8, 5);
        
        assert_eq!(
            parse_time_arg(Some(time), default).unwrap(),
            expected
        );
    }

    #[test]
    fn fails_when_hours_gt_23() {
        let time = "24:00";
        
        assert!(
            parse_time_arg(Some(time), Local::now()).is_err(),
            "should not be able to parse hours greater than 24"
        );
    }

    #[test]
    fn fails_when_hours_not_numeric() {
        let time = "hello:00";
        
        assert!(
            parse_time_arg(Some(time), Local::now()).is_err(),
            "should not be able to parse non-numeric hours"
        );
    }
}