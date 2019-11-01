use scraper::{Html, Selector};

// 58px = 1 hour
// 60 / 58 = 1.03448275862 (minutes per px)
// 60 / 58 * 60 = 62.0689655172 (seconds per px)
const SECONDS_PER_WIDTH_PX: f64 = 62.0689655172;

fn main() -> Result<(), reqwest::Error> {
    // println!("Hello, world!");

    let html = reqwest::get("https://industryrinostation.roomzilla.net/")?
        .text()?;

    // println!("{}", html);

    let document = Html::parse_document(&html);
    let selector = Selector::parse("div.reserved").unwrap();

    for element in document.select(&selector) {
        // assert_eq!("table", element.value().name());
        // <div class='res_16699828 reserved before_now_reserved tip' day='2019-11-01' reservation_id='16699828' room_keyname='ne1' room_name='NE1' seconds='37800.0' style='width: 58px;' tooltip='<strong>Convercent Team discussion</strong><br>10:30 am - 11:30 am<br>Laura Ling'></div>
        println!("day={}, room={}, seconds={}", element.value().attr("day").unwrap(), element.value().attr("room_name").unwrap(), element.value().attr("seconds").unwrap());
        let day = element.value().attr("day").unwrap();
        let room = element.value().attr("room_name").unwrap();
        let start = element.value().attr("seconds").unwrap();
        let style = element.value().attr("style").unwrap();
        let duration = (&style[7..style.find("px;").unwrap()].parse::<f64>().unwrap() * SECONDS_PER_WIDTH_PX).round();
        println!("{}, {}", start, duration);
    }

    Ok(())
}
