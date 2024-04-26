use indoc::formatdoc;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use time::{OffsetDateTime, PrimitiveDateTime};

// Let's work with a string restaurant ID
// And fetch availability

pub const OPENTABLE_URL: &str = "https://www.opentable.com/";
pub const RESTAURANT_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=RestaurantsAvailability";
pub const EXPERIENCE_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=ExperienceAvailability";
pub const BOOK_DETAILS_EXPERIENCE_SLOT_LOCK_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=mutation&opname=BookDetailsExperienceSlotLock";
pub const MAKE_RESERVATION_URL: &str = "https://www.opentable.com/dapi/booking/make-reservation";
pub const SPREEDLY_ENVIRONMENT_KET: &str = "BZiZWqR6ai03EW7Ep7sMIwaB4TI";

pub async fn obtain_csrf_token(
    client: &Client
) -> Result<String, anyhow::Error> {

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
    date_time: &time::PrimitiveDateTime,
    backward_minutes: u32,
    forward_minutes: u32,
    forward_days: u32
) -> Result<Response, anyhow::Error>{

    let date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let date_time_str = date_time.format(&date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_date_time_str = date_time.format(&referer_date_time_format)?;

    let referer_str = format!("https://www.opentable.com/booking/experiences-availability?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        restaurant_id,
        experience_id,
        party_size,
        referer_date_time_str
    );

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
) -> Result<Response, anyhow::Error> {

    let reservation_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let reservation_date_time_str = reservation_date_time.format(&reservation_date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_date_time_str = reservation_date_time.format(&referer_date_time_format)?;

    let referer_str = format!("https://www.opentable.com/booking/experiences-seating-options?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        restaurant_id,
        experience_id,
        party_size,
        referer_date_time_str
    );

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

    client.post(BOOK_DETAILS_EXPERIENCE_SLOT_LOCK_URL)
    .header("accept", "*/*")
    .header("accept-language", "en-US,en;q=0.9")
    .header("content-type", "application/json")
    .header("cookie", "")
    .header("origin", "https://www.opentable.com")
    .header("ot-page-group", "booking")
    .header("ot-page-type", "experiences_seating_options")
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
    .header("x-query-timeout", "4000")
    .body(body)
        .send()
        .await
        .map_err(|e| e.into())
}


pub struct MakeReservationDetails<'a> {
    pub attribution_token: &'a str,
    pub credit_card_last_four: &'a str,
    pub credit_card_mmyy: &'a str,
    pub credit_card_token: &'a str,
    pub dining_area_id: u32,
    pub email: &'a str,
    pub experience_id: u32,
    pub experience_version: u32,
    pub fbp: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub party_size: u32,
    pub points: u32,
    pub points_type: &'a str,
    pub reservation_attribute: &'a str,
    pub reservation_date_time: &'a time::PrimitiveDateTime,
    pub reservation_type: &'a str,
    pub restaurant_id: u32,
    pub slot_availability_token: &'a str,
    pub slot_hash: u32,
    pub slot_lock_id: u32,
    pub phone_number: &'a str
}
// // TODO: Look up how people typically pass this many
// // arguments when making a request w/ JSON data
// pub async fn make_experience_reservation<'a>(
//     client: &Client,
//     make_reservation_details: MakeReservationDetails<'a>
// ) -> Result<Response, anyhow::Error> {
    
//     let reservation_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
//     let reservation_date_time_str = make_reservation_details.reservation_date_time.format(&reservation_date_time_format)?;
//     let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
//     let referer_date_time_str = make_reservation_details.reservation_date_time.format(&referer_date_time_format)?;

//     // let current_date_time = time::OffsetDateTime::from(std::time::SystemTime::now());
//     // prin
//     // let attribution_token__current_date_time_format = time::format_description::parse("[][][]")?;

//     let attribution_token_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]%3A[minute]%3A[second]")?;
//     let attribution_token_date_time_str = make_reservation_details.reservation_date_time.format(&attribution_token_date_time_format)?;

//     let referer_str = format!("https://www.opentable.com/booking/experiences-details?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
//         make_reservation_details.restaurant_id,
//         make_reservation_details.experience_id,
//         make_reservation_details.party_size,
//         referer_date_time_str
//     );

//     let attribution_token_str = format!(
//         "x={}&c=1&pt1=1&pt2=1&er={}&p1ca=booking%2Fexperiences-availability&p1q=rid%3D{}%26experienceId%3D{}%26modal%3Dtrue%26covers%3D{}%26dateTime%3D2024-05-06T19%3A00%3A00",
//         attribution_token_date_time_str,
//         make_reservation_details.restaurant_id,
//         make_reservation_details.restaurant_id,
//         make_reservation_details.experience_id,
//         make_reservation_details.party_size,

//     );

//     // I think credit card provider depends on country.
//     // For now, I think the US will do!
//     let body = formatdoc!(
//         r#"{{
//             "additionalServiceFees":[],
//             "attributionToken":"{attribution_token}",
//             "country":"US",
//             "creditCardLast4":"{credit_card_last_four}",
//             "creditCardMMYY":"{credit_card_mmyy}",
//             "creditCardProvider":"spreedly",
//             "creditCardToken":"{credit_card_token}",
//             "diningAreaId":{dining_area_id},
//             "email":"{email}",
//             "experienceAddOns":[],
//             "experienceId":{experience_id},
//             "experienceVersion":{experience_version},
//             "fbp":"{fbp}",
//             "firstName":"{first_name}",
//             "isBookAnywhere":true,
//             "isModify":false,
//             "katakanaFirstName":"",
//             "katakanaLastName":"",
//             "lastName":"{last_name}",
//             "nonBookableExperiences":[],
//             "partySize":{party_size},
//             "points":{points},
//             "pointsType":"{points_type}",
//             "reservationAttribute":"{reservation_attribute}",
//             "reservationDateTime":"{reservation_date_time}",
//             "reservationType":"{reservation_type}",
//             "restaurantId":{restaurant_id},
//             "scaRedirectUrl":"https://www.opentable.com/booking/payments-sca",
//             "slotAvailabilityToken":"{slot_availability_token}",
//             "slotHash":{slot_hash},
//             "slotLockId": {slot_lock_id},
//             "tipAmount":0,
//             "tipPercent":0,
//             "phoneNumber":"{phone_number}",
//             "phoneNumberCountryId":"US",
//             "optInEmailRestaurant":false
//         }}"#,
//         attribution_token = make_reservation_details.attribution_token,
//         credit_card_last_four = make_reservation_details.credit_card_last_four,
//         credit_card_mmyy = make_reservation_details.credit_card_mmyy,
//         credit_card_token = make_reservation_details.credit_card_token,
//         dining_area_id = make_reservation_details.dining_area_id,
//         email = make_reservation_details.email,
//         experience_id = make_reservation_details.experience_id,
//         experience_version = make_reservation_details.experience_version,
//         fbp = make_reservation_details.fbp,
//         first_name = make_reservation_details.first_name,
//         last_name = make_reservation_details.last_name,
//         party_size = make_reservation_details.party_size,
//         points = make_reservation_details.points,
//         points_type = make_reservation_details.points_type,
//         reservation_attribute = make_reservation_details.reservation_attribute,
//         reservation_date_time = reservation_date_time_str,
//         reservation_type = make_reservation_details.reservation_type,
//         restaurant_id = make_reservation_details.restaurant_id,
//         slot_availability_token = make_reservation_details.slot_availability_token,
//         slot_hash = make_reservation_details.slot_hash,
//         slot_lock_id = make_reservation_details.slot_lock_id,
//         phone_number = make_reservation_details.phone_number
//     );

//     client.post()
//         .header("accept", "*/*")
//         .header("accept-language", "en-US,en;q=0.9")
//         .header("content-type", "application/json")
//         .header("cookie", "")
//         .header("origin", "https://www.opentable.com")
//         .header("ot-page-group", "booking")
//         .header("ot-page-type", "experiences_availability")
//         .header("priority", "u=1, i")
//         .header("referer", referer_str)
//         .header("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"")
//         .header("sec-ch-ua-mobile", "?0")
//         .header("sec-ch-ua-platform", "\"macOS\"")
//         .header("sec-fetch-dest", "empty")
//         .header("sec-fetch-mode", "cors")
//         .header("sec-fetch-site", "same-origin")
//         .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
//         .header("x-csrf-token", "")
//         .send()
//         .await
//         .map_err(|e| e.into())
// }

pub fn unix_offset_date_time_now_local() -> Result<time::OffsetDateTime, anyhow::Error> {
    let mut timezone: libc::tm = unsafe { std::mem::zeroed() };
    let mut timestamp: libc::time_t = 0;
    unsafe {
        libc::time(&mut timestamp);
        libc::localtime_r(&timestamp, &mut timezone);
    }

    let utc_offset_seconds = i32::try_from(timezone.tm_gmtoff)?;
    let utc_offset = time::UtcOffset::from_whole_seconds(utc_offset_seconds)?;

    let current_date_time_utc = time::OffsetDateTime::from_unix_timestamp(timestamp)?;
    let current_date_time_offset = current_date_time_utc.to_offset(utc_offset);

    Ok(current_date_time_offset)
}