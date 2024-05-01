use anyhow::Context;
use clap::Parser;
use indoc::formatdoc;
use ratatui::layout::Offset;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use serde::Deserialize;
use tokio::runtime;
use std::ops::Sub;
use time::OffsetDateTime;
use std::collections::BTreeMap;

pub const OPENTABLE_URL: &str = "https://www.opentable.com/";
pub const RESTAURANT_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=RestaurantsAvailability";
pub const EXPERIENCE_AVAILABILITY_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=query&opname=ExperienceAvailability";
pub const BOOK_DETAILS_EXPERIENCE_SLOT_LOCK_URL: &str = "https://www.opentable.com/dapi/fe/gql?optype=mutation&opname=BookDetailsExperienceSlotLock";
pub const MAKE_RESERVATION_URL: &str = "https://www.opentable.com/dapi/booking/make-reservation";

pub const SPREEDLY_PAYMENT_METHODS_URL: &str = "https://core.spreedly.com/v1/payment_methods/restricted.json?from=iframe&v=1.124";
pub const SPREEDLY_ENVIRONMENT_KEY: &str = "BZiZWqR6ai03EW7Ep7sMIwaB4TI";

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    details_path: std::path::PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct RunDetails {
    release_date_time: String,
    experience_details: RunExperienceDetails,
    user_details: RunUserDetails,
    // Require a card for now!
    card_details: RunCardDetails,
    fbp: String
}

#[derive(Deserialize, Debug)]
pub struct RunExperienceDetails {
    restaurant_id: u32,
    experience_id: u32,
    experience_version: u32,
    reference_date_time: String,
    forward_days: i64,
    forward_minutes: i64,
    backward_minutes: i64,
    party_size: u32
}

#[derive(Deserialize, Debug)]
pub struct RunUserDetails {
    first_name: String,
    last_name: String,
    email: String,
    phone_number: String
}

#[derive(Deserialize, Debug)]
pub struct RunCardDetails {
    number: String,
    cvv: String,
    expiration_date: String,
    zip_code: String
}

pub async fn run() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let json_run_details_str = std::fs::read_to_string(&args.details_path)?;
    let run_details = serde_json::from_str::<RunDetails>(&json_run_details_str)?;

    println!("run_details: {:#?}", run_details);
    
    let run_experience_details = &run_details.experience_details;
    let run_user_details = &run_details.user_details;
    let run_card_details =&run_details.card_details;

    // TODO: Something cleaner and less confusing
    // BUT this works for now!
    let release_date_time_offset_free = time::OffsetDateTime::parse(
        &run_details.release_date_time,
        &time::format_description::well_known::Iso8601::DEFAULT
    )?;
    
    sleep_til(release_date_time_offset_free).await?;

    // 1. Obtain experience availability. Try until non-empty or exhausted number of attempts.
    let client = reqwest::Client::new();

    let reference_date_time = time::OffsetDateTime::parse(
        &run_details.experience_details.reference_date_time,
        &time::format_description::well_known::Iso8601::DEFAULT
    )?;

    let fetch_experience_availability_details = FetchExperienceAvailabilityDetails {
        restaurant_id: run_experience_details.restaurant_id, 
        experience_id: run_experience_details.experience_id, 
        party_size: run_experience_details.party_size, 
        reference_date_time: reference_date_time.clone(), 
        backward_minutes: run_experience_details.backward_minutes, 
        forward_minutes: run_experience_details.forward_minutes, 
        forward_days: run_experience_details.forward_days,
    };

    let mut day_offset_to_experience_slots = BTreeMap::new();
    let mut fetch_tries_limit = 200;

    // If one of these is false we want to fail!!!
    while day_offset_to_experience_slots.is_empty() && fetch_tries_limit > 0 {
        let fetch_experience_availability_response = fetch_experience_availability(
            &client, 
            &fetch_experience_availability_details
        ).await?;
    
        let deser_fetch_experience_availability_response = fetch_experience_availability_response.json::<FetchExperienceAvailabilityResponse>().await?;

        day_offset_to_experience_slots = available_experience_slots(deser_fetch_experience_availability_response);

        fetch_tries_limit -= 1;
    }

    if day_offset_to_experience_slots.is_empty() {
        return Err(anyhow::anyhow!("Could not obtain available_experience slots!"))
    }

    // println!("day_offset_to_experience_slots: {:#?}", day_offset_to_experience_slots);

    // 2. Lock the first available slot!
    let locked_slot = lock_first_available_slot(
        &client, 
        &day_offset_to_experience_slots, 
        &LockFirstAvailableSlotDetails {
            restaurant_id: run_experience_details.restaurant_id,
            reference_date_time,
            party_size: run_experience_details.party_size,
            experience_id: run_experience_details.experience_id,
            experience_version: run_experience_details.experience_version
        }
    )
    .await?
    .context("Failed to lock a slot")?;

    println!("locked_slot: {:#?}", locked_slot);

    // 3. Add card to Spreedly
    // TODO: Add support for non card requiring experiences
    // But for not throw an error on not having card_details

    let date_format = time::macros::format_description!("[year]-[month]-[day]");
    let expiration_date = time::Date::parse(&run_card_details.expiration_date, &date_format)?;

    let card_details = SpreedlyAddPaymentMethodDetails {
        number: &run_card_details.number,
        cvv: &run_card_details.cvv,
        first_name: &run_user_details.first_name,
        last_name: &run_user_details.last_name,
        month: expiration_date.month() as u32,
        year: expiration_date.year() as u32,
        zip_code: &run_card_details.zip_code
    };

    let spreedly_add_payment_method_response = spreedly_add_payment_method(
        &client, 
        &card_details
    ).await?;

    println!("spreedly_add_payment_method_response: {:#?}", spreedly_add_payment_method_response);

    let deser_spreedly_add_payment_method_response = spreedly_add_payment_method_response.json::<SpreedlyAddPaymentMethodResponse>().await?;

    println!("deser_spreedly_add_payment_method_response: {:#?}", deser_spreedly_add_payment_method_response);

    // TODO: CHECK FOR CARD ADDED SUCCESS IN REAL FLOWS
    if !deser_spreedly_add_payment_method_response.transaction.succeeded {
        return Err(anyhow::anyhow!("Failed to add payment method!"));
    }

    // 4. Make the reservation!
    let split_pos = run_card_details.number
        .char_indices()
        .nth_back(3)
        .context("CC number does not have enough digits!")?
        .0;
    let credit_card_last_four = &run_card_details.number[split_pos..];

    println!("Credit Card last four: {}", credit_card_last_four);

    let mmyy_format = time::macros::format_description!("[month][year repr:last_two]");
    let credit_card_mmyy = expiration_date.format(&mmyy_format)?;

    println!("Credit Card mmyy: {}", credit_card_mmyy);

    let make_experience_reservation_details = MakeExperienceReservationDetails{
        credit_card_last_four,
        credit_card_mmyy: &credit_card_mmyy,
        credit_card_token: &deser_spreedly_add_payment_method_response.transaction.payment_method.token,
        dining_area_id: 1,
        email: &run_user_details.email,
        experience_id: run_experience_details.experience_id,
        experience_version: run_experience_details.experience_version,
        fbp: &run_details.fbp,
        first_name: &run_user_details.first_name,
        last_name: &run_user_details.last_name,
        party_size: run_experience_details.party_size,
        points: locked_slot.experience_slot.points_value,
        points_type: &locked_slot.experience_slot.points_type,
        reservation_attribute: &locked_slot.chosen_attribute,
        reservation_date_time: &locked_slot.date_time,
        restaurant_id: run_experience_details.restaurant_id,
        slot_availability_token: &locked_slot.experience_slot.slot_availability_token,
        slot_hash: locked_slot.experience_slot.slot_hash,
        slot_lock_id: locked_slot.slot_lock_id,
        phone_number: &run_user_details.phone_number
    };

    let make_experience_reservation_response = make_experience_reservation(
        &client,
        &make_experience_reservation_details
    )
    .await?;

    println!("make_experience_reservation_response: {:#?}", make_experience_reservation_response);

    let make_experience_reservation_response_json = make_experience_reservation_response.json::<serde_json::Value>().await?;

    println!("make_experience_reservation_response_json: {:#?}", make_experience_reservation_response_json);

    Ok(())
}

pub async fn sleep_til(
    release_date_time_offset_free: time::OffsetDateTime
) -> Result<(), anyhow::Error> {
    let current_date_time_local = unix_offset_date_time_now_local()?;
    let release_date_time_local = release_date_time_offset_free.replace_offset(current_date_time_local.offset());
    println!("current_date_time_local: {}", current_date_time_local);
    println!("release_date_time_local: {}", release_date_time_local);

    let sleep_duration = release_date_time_local.sub(current_date_time_local);
    tokio::time::sleep(sleep_duration.unsigned_abs()).await;

    Ok(())
}

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

// TODO: Consider a more general type
#[derive(Deserialize, Debug)]
pub struct FetchExperienceAvailabilityResponse {
    pub data: ExperienceAvailabilityData
}

#[derive(Deserialize, Debug)]
pub struct ExperienceAvailabilityData {
    #[serde(rename(deserialize="experienceAvailability"))]
    pub experience_availability: ExperienceAvailabilityResponse
}

#[derive(Deserialize, Debug)]
pub struct ExperienceAvailabilityResponse {
    pub available: Vec<ExperienceAvailability>
}

#[derive(Deserialize, Debug)]
pub struct ExperienceAvailability {
    #[serde(rename(deserialize="dayOffset"))]
    pub day_offset: i64,
    #[serde(rename(deserialize="restaurantSet"))]
    pub restaurant_set: Vec<RestaurantSet>,
}

#[derive(Deserialize, Debug)]
pub struct RestaurantSet {
    pub available: bool,
    pub results: RestaurantSetResults
}

#[derive(Deserialize, Debug)]
pub struct RestaurantSetResults {
    pub experience: Option<Vec<ExperienceSlot>>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExperienceSlot {
    pub attributes: Vec<String>,
    #[serde(rename(deserialize="bookableExperienceDiningAreas"))]
    pub bookable_experience_dining_areas: Vec<BookableExperienceDiningAreas>,
    #[serde(rename(deserialize="creditCardRequired"))]
    pub credit_card_required: bool,
    #[serde(rename(deserialize="pointsType"))]
    pub points_type: String,
    #[serde(rename(deserialize="pointsValue"))]
    pub points_value: u32,
    #[serde(rename(deserialize="slotAvailabilityToken"))]
    pub slot_availability_token: String,
    #[serde(rename(deserialize="slotHash"))]
    pub slot_hash: u64,
    #[serde(rename(deserialize="timeOffsetMinutes"))]
    pub time_offset_minutes: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BookableExperienceDiningAreas {
    #[serde(rename(deserialize="diningAreas"))]
    pub dining_areas: Vec<DiningArea>
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiningArea {
    pub attributes: Vec<String>,
    #[serde(rename(deserialize="diningAreaId"))]
    pub dining_area_id: u32
}

#[derive(Debug)]
pub struct FetchExperienceAvailabilityDetails {
    pub restaurant_id: u32,
    pub experience_id: u32,
    pub party_size: u32,
    pub reference_date_time: time::OffsetDateTime,
    pub backward_minutes: i64,
    pub forward_minutes: i64,
    pub forward_days: i64
}

pub async fn fetch_experience_availability(
    client: &Client,
    details: &FetchExperienceAvailabilityDetails
) -> Result<Response, anyhow::Error>{
    let date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let reference_date_time_str = details.reference_date_time.format(&date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_reference_date_time_str = details.reference_date_time.format(&referer_date_time_format)?;

    let referer_str = format!("https://www.opentable.com/booking/experiences-availability?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        details.restaurant_id,
        details.experience_id,
        details.party_size,
        referer_reference_date_time_str
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
        restaurant_id = details.restaurant_id,
        party_size = details.party_size,
        date_time_str = reference_date_time_str,
        experience_id = details.experience_id,
        backward_minutes = details.backward_minutes,
        forward_minutes = details.forward_minutes,
        forward_days = details.forward_days
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

// Filter out out all bookable experience availablity slots.
// Mind day and time offsets!
pub fn available_experience_slots<'a>(
    response: FetchExperienceAvailabilityResponse
) -> BTreeMap<i64, Vec<ExperienceSlot>> {
    // Think a vec would be fine but for consistencies sake let's go with BTreeMap
    let mut day_offset_to_experience_slots = BTreeMap::new();

    for experience_availability in response.data.experience_availability.available {

        for restaurant_set in experience_availability.restaurant_set {
            if restaurant_set.available {
                if let Some(experience_slots) = restaurant_set.results.experience {
                    day_offset_to_experience_slots.insert(
                        experience_availability.day_offset, 
                        experience_slots
                    );
                }
            } 
        }
    }

    day_offset_to_experience_slots
}

pub struct LockFirstAvailableSlotDetails {
    pub restaurant_id: u32,
    pub reference_date_time: time::OffsetDateTime,
    pub party_size: u32,
    pub experience_id: u32,
    pub experience_version: u32,
}

#[derive(Debug)]
pub struct LockedSlot {
    pub slot_lock_id: u64,
    pub experience_slot: ExperienceSlot,
    pub date_time: OffsetDateTime,
    pub chosen_attribute: String
}

// TODO: Consider whether or not closely couple date time
// w/ the available slots (the offsets are relative to given date time)
pub async fn lock_first_available_slot<'a>(
    client: &Client,
    day_offset_to_experience_slots: &BTreeMap<i64, Vec<ExperienceSlot>>,
    details: &LockFirstAvailableSlotDetails
) -> Result<Option<LockedSlot>, anyhow::Error> {
    for (day_offset, experience_slots) in day_offset_to_experience_slots.iter() {
        let day_offset_reference_date_time = details
            .reference_date_time
            .checked_add(time::Duration::days(*day_offset))
            .context("Adding day_offset to reference_date_time failed")?;
        for experience_slot in experience_slots.iter() {



            let reservation_date_time = day_offset_reference_date_time
                .checked_add(time::Duration::minutes(experience_slot.time_offset_minutes))
                .context("Adding time_offset_minutes to day_offset_reference_date_time failed")?;

            let dining_area = experience_slot
                .bookable_experience_dining_areas
                .get(0)
                .context("bookableExperienceDiningAreas is empty")?
                .dining_areas
                .get(0)
                .context("diningAreas is empty")?;

            let seating_option = dining_area
                .attributes
                .get(0)
                .context("Experience has no attributes")?
                .to_uppercase();

            let dining_area_id = dining_area.dining_area_id;

            let book_experience_details = BookExperienceDetails {
                restaurant_id: details.restaurant_id,
                seating_option: seating_option.clone(),
                reservation_date_time,
                party_size: details.party_size,
                slot_hash: experience_slot.slot_hash,
                experience_id: details.experience_id,
                experience_version: details.experience_version,
                dining_area_id
            };

            // println!("attempting to lock slot w/: {:#?}", book_experience_details);

            let response= execute_book_details_experience_slot_lock(
                client,
                &book_experience_details
            ).await?;

            // println!("slot lock response: {:#?}", response);

            let deser_response = response.json::<ExecuteBookDetailSlotLockResponse>().await?;

            // println!("slot lock deser response: {:#?}", deser_response);

            let slot_lock_response = deser_response.data.lock_experience_slot;
            if slot_lock_response.success {
                return Ok(
                    Some(
                        LockedSlot {
                            slot_lock_id: slot_lock_response.slot_lock.slot_lock_id,
                            experience_slot: experience_slot.clone(),
                            date_time: reservation_date_time,
                            chosen_attribute: seating_option
                        }
                    )
                );
            }
        }
    }
    Ok(None)
}

#[derive(Deserialize, Debug)]
pub struct ExecuteBookDetailSlotLockResponse {
    data: ExecuteBookDetailSlotLockData 
}

#[derive(Deserialize, Debug)]
pub struct ExecuteBookDetailSlotLockData {
    #[serde(rename(deserialize="lockExperienceSlot"))]
    lock_experience_slot: SlotLockResponse
}

#[derive(Deserialize, Debug)]
pub struct SlotLockResponse {
    #[serde(rename(deserialize="slotLock"))]
    slot_lock: SlotLock,
    success: bool
}

#[derive(Deserialize, Debug)]
pub struct SlotLock {
    #[serde(rename(deserialize="slotLockId"))]
    slot_lock_id: u64
}

#[derive(Debug)]
pub struct BookExperienceDetails {
    pub restaurant_id: u32,
    pub seating_option: String,
    pub reservation_date_time: time::OffsetDateTime,
    pub party_size: u32,
    pub slot_hash: u64,
    pub experience_id: u32,
    pub experience_version: u32,
    pub dining_area_id: u32,
}

pub async fn execute_book_details_experience_slot_lock(
    client: &Client,
    book_experience_details: &BookExperienceDetails
) -> Result<Response, anyhow::Error> {

    let reservation_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let reservation_date_time_str = book_experience_details.reservation_date_time.format(&reservation_date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_date_time_str = book_experience_details.reservation_date_time.format(&referer_date_time_format)?;

    let referer_str = format!("https://www.opentable.com/booking/experiences-seating-options?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        book_experience_details.restaurant_id,
        book_experience_details.experience_id,
        book_experience_details.party_size,
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
        restaurant_id = book_experience_details.restaurant_id,
        seating_option = book_experience_details.seating_option,
        reservation_date_time_str = reservation_date_time_str,
        party_size = book_experience_details.party_size,
        slot_hash = book_experience_details.slot_hash,
        experience_id = book_experience_details.experience_id,
        experience_version = book_experience_details.experience_version,
        dining_area_id = book_experience_details.dining_area_id
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

pub struct SpreedlyAddPaymentMethodDetails<'a> {
    pub number: &'a str,
    pub cvv: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub month: u32,
    pub year: u32,
    pub zip_code: &'a str // Maybe number type idk yet
}

#[derive(Deserialize, Debug)]
pub struct SpreedlyAddPaymentMethodResponse {
    pub transaction: SpreedlyTransaction
}

#[derive(Deserialize, Debug)]
pub struct SpreedlyTransaction {
    pub payment_method: SpreedlyPaymentMethod,
    pub succeeded: bool
}

#[derive(Deserialize, Debug)]
pub struct SpreedlyPaymentMethod {
    pub token: String // This is the token we need!
}

// Assumes NA
pub async fn spreedly_add_payment_method<'a>(
    client: &Client,
    card_details: &SpreedlyAddPaymentMethodDetails<'a>
) -> Result<Response, anyhow::Error> {
    let body = formatdoc!(
        r#"{{
            "environment_key":"{environment_key}",
            "payment_method":{{
                "credit_card":{{
                    "number":"{number}",
                    "verification_value":"{verification_value}",
                    "full_name":"{first_name} {last_name}",
                    "month":"{month}",
                    "year":"{year}",
                    "zip":"{zip}"
                }}
            }}
        }}"#,
        environment_key = SPREEDLY_ENVIRONMENT_KEY,
        number = card_details.number,
        verification_value = card_details.cvv,
        first_name = card_details.first_name,
        last_name = card_details.last_name,
        month = card_details.month,
        year = card_details.year,
        zip = card_details.zip_code
    );

    client.post(SPREEDLY_PAYMENT_METHODS_URL)
        .header("accept", "*/*")
        .header("accept-language", "en-US,en;q=0.9")
        .header("content-type", "application/json")
        .header("origin", "https://core.spreedly.com")
        .header("priority", "u=1, i")
        .header("referer", "https://core.spreedly.com/v1/embedded/number-frame-1.124.html")
        .header("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header("spreedly-environment-key", SPREEDLY_ENVIRONMENT_KEY)
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .body(body)
        .send()
        .await
        .map_err(|e| e.into())
}

pub struct MakeExperienceReservationDetails<'a> {
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
    pub reservation_date_time: &'a time::OffsetDateTime,
    pub restaurant_id: u32,
    pub slot_availability_token: &'a str,
    pub slot_hash: u64,
    pub slot_lock_id: u64,
    pub phone_number: &'a str,
}



// TODO: Look up how people typically pass this many
// arguments when making a request w/ JSON data
pub async fn make_experience_reservation<'a>(
    client: &Client,
    make_reservation_details: &MakeExperienceReservationDetails<'a>
) -> Result<Response, anyhow::Error> {
    
    let reservation_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]")?;
    let reservation_date_time_str = make_reservation_details.reservation_date_time.format(&reservation_date_time_format)?;
    let referer_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")?;
    let referer_date_time_str = make_reservation_details.reservation_date_time.format(&referer_date_time_format)?;

    // 2024-04-24T18%3A57%3A21
    // 2024-05-06T19%3A00%3A00
    let current_date_time = unix_offset_date_time_now_local()?;
    let attribution_token_date_time_format = time::format_description::parse("[year]-[month]-[day]T[hour]%3A[minute]%3A[second]")?;
    let attribution_token_current_date_time_str = current_date_time.format(&attribution_token_date_time_format)?;
    let attribution_token_reservation_date_time_str = make_reservation_details.reservation_date_time.format(&attribution_token_date_time_format)?;

    let referer_str = format!("https://www.opentable.com/booking/experiences-details?rid={}&experienceId={}&modal=true&covers={}&dateTime={}", 
        make_reservation_details.restaurant_id,
        make_reservation_details.experience_id,
        make_reservation_details.party_size,
        referer_date_time_str
    );

    let attribution_token_str = format!(
        "x={}&c=1&pt1=1&pt2=1&er={}&p1ca=booking%2Fexperiences-availability&p1q=rid%3D{}%26experienceId%3D{}%26modal%3Dtrue%26covers%3D{}%26dateTime%3D{}",
        attribution_token_current_date_time_str,
        make_reservation_details.restaurant_id,
        make_reservation_details.restaurant_id,
        make_reservation_details.experience_id,
        make_reservation_details.party_size,
        attribution_token_reservation_date_time_str
    );

    println!("forming make_experience_reservation_body!!");

    // I think credit card provider depends on country.
    // For now, I think the US will do!
    let body = formatdoc!(
        r#"{{
            "additionalServiceFees":[],
            "attributionToken":"{attribution_token}",
            "country":"US",
            "creditCardLast4":"{credit_card_last_four}",
            "creditCardMMYY":"{credit_card_mmyy}",
            "creditCardProvider":"spreedly",
            "creditCardToken":"{credit_card_token}",
            "diningAreaId":{dining_area_id},
            "email":"{email}",
            "experienceAddOns":[],
            "experienceId":{experience_id},
            "experienceVersion":{experience_version},
            "fbp":"{fbp}",
            "firstName":"{first_name}",
            "isBookAnywhere":true,
            "isModify":false,
            "katakanaFirstName":"",
            "katakanaLastName":"",
            "lastName":"{last_name}",
            "nonBookableExperiences":[],
            "partySize":{party_size},
            "points":{points},
            "pointsType":"{points_type}",
            "reservationAttribute":"{reservation_attribute}",
            "reservationDateTime":"{reservation_date_time}",
            "reservationType":"Experience",
            "restaurantId":{restaurant_id},
            "scaRedirectUrl":"https://www.opentable.com/booking/payments-sca",
            "slotAvailabilityToken":"{slot_availability_token}",
            "slotHash":{slot_hash},
            "slotLockId": {slot_lock_id},
            "tipAmount":0,
            "tipPercent":0,
            "phoneNumber":"{phone_number}",
            "phoneNumberCountryId":"US",
            "optInEmailRestaurant":false
        }}"#,
        attribution_token = attribution_token_str,
        credit_card_last_four = make_reservation_details.credit_card_last_four,
        credit_card_mmyy = make_reservation_details.credit_card_mmyy,
        credit_card_token = make_reservation_details.credit_card_token,
        dining_area_id = make_reservation_details.dining_area_id,
        email = make_reservation_details.email,
        experience_id = make_reservation_details.experience_id,
        experience_version = make_reservation_details.experience_version,
        fbp = make_reservation_details.fbp,
        first_name = make_reservation_details.first_name,
        last_name = make_reservation_details.last_name,
        party_size = make_reservation_details.party_size,
        points = make_reservation_details.points,
        points_type = make_reservation_details.points_type,
        reservation_attribute = make_reservation_details.reservation_attribute,
        reservation_date_time = reservation_date_time_str,
        restaurant_id = make_reservation_details.restaurant_id,
        slot_availability_token = make_reservation_details.slot_availability_token,
        slot_hash = make_reservation_details.slot_hash,
        slot_lock_id = make_reservation_details.slot_lock_id,
        phone_number = make_reservation_details.phone_number
    );

    println!("Making Reservation Post Request!!");

    client.post(MAKE_RESERVATION_URL)
        .body(body)
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
        .send()
        .await
        .map_err(|e| e.into())
}


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