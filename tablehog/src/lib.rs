use indoc::formatdoc;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use time::PrimitiveDateTime;

// Let's work with a string restaurant ID
// And fetch availability

pub const OPENTABLE_URL: &str = "https://www.opentable.com/";
pub const RESTAURANT_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=RestaurantsAvailability";
pub const EXPERIENCE_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=ExperienceAvailability";

// pub async fn fetch_restaurant_availability(
//     client: &Client,
//     restaurant_id: u32,
//     party_size: u32,
//     date: &time::Date,
//     time: &time::Time    
// ) -> Result<(), anyhow::Error> {
//     let date_format = time::format_description::parse("[year]-[month]-[day]")?;
//     let date_str = date.format(&date_format)?;

//     let time_format = time::format_description::parse("[hour]:[minute]")?;
//     let time_str = time.format(&time_format)?;

//     let body = serde_json::json!({
//         "operationName":"ExperienceAvailability", 
//         "variables":{
//             "onlyPop":false,
//             "forwardDays":0,
//             "requireTimes":false,
//             "requireTypes":[],
//             "restaurantIds":[restaurant_id],
//             "date":date_str,
//             "time":time_str,
//             "partySize":party_size,
//             "databaseRegion":"NA",
//             "restaurantAvailabilityTokens":[null],
//             "slotDiscovery":["on"],
//             "loyaltyRedemptionTiers":[],
//             "attributionToken":null
//         },
//         "extensions":{
//             "persistedQuery":{
//                 "version":1,
//                 "sha256Hash":"2aee2372b4496d091f057a6004c6d79fbf01ffdc8faf13d3887703a1ba45a3b8"
//             }
//         }
//     });


//     // Make the POST request
//     let mut builder = client.post(RESTAURANT_AVAILABILITY_URL)
//         .json(
//             &body
//         );

//     Ok(())
// }

pub async fn obtain_csrf_token(
    client: &Client
) -> Result<String, anyhow::Error> {

    // let cookie = "otuvid=6121D209-FDB2-497E-BB44-38B7B6C9F74F; _ga_Y77FR7F6XF=GS1.1.1713927421.15.1.1713927455.0.0.0";

    println!("obtaining CSRF token");
    let response = client.get(OPENTABLE_URL)
        .header("user-agent", "curl/7.87.0")
        .header("accept", "*/*")
        .send()
        .await?;

    println!("site request response:\n{:#?}", response);
       
    let html = response.text()
        .await?;

    println!("html:\n{}", html);

    let document = Html::parse_document(&html);

    let selector = Selector::parse("script")
        .map_err(|_| anyhow::anyhow!("Failed to parse html for 'script' tags"))?;

    for script in document.select(&selector) {
        let script_content = script.inner_html();

        for line in script_content.lines() {
            if line.contains("window.__CSRF_TOKEN__") {
                // Splits into two at the '=' sign
                if let Some(token) = line.split("=").nth(1) {
                    return Ok(token.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("No CSRF token present in HTML"))
}

pub async fn fetch_experience_availability(
    client: &Client,
    restaurant_id: u32,
    experience_id: u32,
    party_size: u32,
    date: &time::Date,
    time: &time::Time,
    backward_minutes: u32,
    forward_minutes: u32,
    forward_days: u32
) -> Result<Response, anyhow::Error>{

    let date_time = time::PrimitiveDateTime::new(date.clone(), time.clone());
    let date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let date_time_str = date_time.format(&date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_date_time_str = date_time.format(&referer_date_time_format)?;

    println!("date_time_str: {}", date_time_str);

    let body = formatdoc!(
        r#"{{
            "operationName":"ExperienceAvailability",
            "variables":{{
                "includeDiningAreas":true,
                "transformOutdoorToDefault":false,
                "restaurantIds":[{restaurant_id}],
                "partySize":{party_size},
                "dateTime":"{date_time_str}",
                "experienceId":{experience_id},
                "type":"Experience",
                "returnTimeSlots":true,
                "backwardMinutes":{backward_minutes},
                "forwardMinutes":{forward_minutes},
                "forwardDays":{forward_days}
            }},
            "extensions":{{
                "persistedQuery":{{
                    "version":1,
                    "sha256Hash":"9a7cd200454543087f0c500e9ac7fd04a811a107c0f8e1eca7f2714cbfeaf4e0"
                }}
            }}
        }}"#,
        restaurant_id = restaurant_id,
        party_size = party_size,
        date_time_str = date_time_str,
        experience_id = experience_id,
        backward_minutes = backward_minutes,
        forward_minutes = forward_minutes,
        forward_days = forward_days
    );

    let referer_str = format!("https://www.opentable.com/booking/experiences-availability?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        restaurant_id,
        experience_id,
        party_size,
        referer_date_time_str
    );
    
    client.post(EXPERIENCE_AVAILABILITY_URL)
        .header("accept", "*/*")
        .header("accept-language", "en-US,en;q=0.9")
        .header("content-type", "application/json")
        .header("cookie", "")
        .header("origin", "https://www.opentable.com")
        .header("ot-page-group", "booking")
        .header("ot-page-type", "experiences_availability")
        .header("priority", "u=1, i")
        .header("referer", referer_str)
        .header("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .header("x-csrf-token", "")
        .header("x-query-timeout", "6883")
        .body(body)
        .send()
        .await
        .map_err(|e| e.into())
}

pub async fn lock_book_details_experience_slot(
    client: &Client,
    restaurant_id: u32,
    seating_option: &str,
    reservation_date_time: &PrimitiveDateTime,
    party_size: u32,
    slot_hash: u32,
    experience_id: u32,
    experience_version: u32,
    dining_area_id: u32
) {

    let reservation_date_time_str = "";

    let body = formatdoc!(
        r#"{{
            "operationName":"BookDetailsExperienceSlotLock",
            "variables":{{
                "experienceSlotLockInput":{{
                    "restaurantId":{restaurant_id},
                    "seatingOption":"{seating_option}",
                    "reservationDateTime":"{reservation_date_time_str}",
                    "partySize":{party_size},
                    "databaseRegion":"NA",
                    "slotHash":{slot_hash},
                    "experienceId":{experience_id},
                    "experienceVersion":{experience_version},
                    "diningAreaId":{dining_area_id}
                }}
            }},
            "extensions":{{
                "persistedQuery":{{
                    "version":1,
                    "sha256Hash":"9d4778c80c7a86c581760ee03ced083866021c4618b1bda4f48912d599bcca26"
                }}
            }}
        }}"#,
        restaurant_id = restaurant_id,
        seating_option = seating_option,
        reservation_date_time_str = reservation_date_time_str,
        party_size = party_size,
        slot_hash = slot_hash,
        experience_id = experience_id,
        experience_version = experience_version,
        dining_area_id = dining_area_id
    );

}