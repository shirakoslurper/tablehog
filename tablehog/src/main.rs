use std::thread::current;

use tablehog::*;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    let current_offset_date_time = unix_offset_date_time_now_local()?;
    println!("{}", current_offset_date_time);


    let client = reqwest::Client::new();

    // let csrf_token = obtain_csrf_token(&client).await?;
    // println!("csrf_token: {}", csrf_token);

    // let restaurant_id = 118267; // Maude
    // let experience_id = 285395; // Maude 10th Anniversary

    let restaurant_id = 14410;  // Osteria Mozza
    let experience_id = 208989; // Pasta Tasting Menu
    // let seating_option = "COUNTER";
    // let slot_hash = 2635132527;
    // let experience_version = 9;
    // let dining_area_id = 1;

    let party_size = 2;
    // let date = time::macros::date!(2024-05-06);
    // let time_ = time::macros::time!(18:30);
    let date_time = time::OffsetDateTime::new_in_offset(
        time::macros::date!(2024-05-06),
        time::macros::time!(18:00), 
        current_offset_date_time.offset()
    );
    let forward_minutes = 240;
    let backwards_minutes= 0;
    let forward_days = 30;

    let experience_availability_response = fetch_experience_availability(
        &client, 
        restaurant_id, 
        experience_id, 
        party_size, 
        &date_time, 
        forward_minutes, 
        backwards_minutes, 
        forward_days,
    ).await?;

    println!("experience_availability_response: \n{:#?}", experience_availability_response);

    let experience_availability_response_json = experience_availability_response.json::<FetchExperienceAvailabilityResponse>().await?;

    println!("experience_availability_response_json: {:#?}", experience_availability_response_json);

    let available_experiences = available_experience_slots(&experience_availability_response_json);

    println!("available_experiences: {:#?}", available_experiences);

    // let slot_lock_response = book_details_experience_slot_lock(
    //     &client, 
    //     restaurant_id, 
    //     seating_option, 
    //     &date_time, 
    //     party_size, 
    //     slot_hash, 
    //     experience_id, 
    //     experience_version, 
    //     dining_area_id
    // ).await?;

    // println!("slot_lock_response: \n{:#?}", slot_lock_response);

    // let slot_lock_response_json = slot_lock_response.json::<serde_json::Value>().await?;

    // println!("slot_lock_response_json: {}", slot_lock_response_json);

    // let card_details = CardDetails {
    //     number: "4347 6970 7337 9635",
    //     cvv: "635",
    //     first_name: "Jack",
    //     last_name: "Baxter",
    //     month: 5,
    //     year: 2026,
    //     zip_code: "90025"
    // };

    // println!("adding card to spreedly");

    // let add_card_to_spreedly_response = spreedly_add_payment_method(
    //     &client, 
    //     &card_details
    // ).await?;

    // println!("add_card_to_spreedly_response: {:#?}", add_card_to_spreedly_response);

    // let add_card_to_spreedly_response_json = add_card_to_spreedly_response.json::<SpreedlyAddPaymentMethodResponse>().await?;

    // println!("add_card_to_spreedly_response_json: {:#?}", add_card_to_spreedly_response_json);

    // let make_reservation_details = MakeReservationDetails{
    //     credit_card_last_four: "",
    //     credit_card_mmyy: "",
    //     credit_card_token: "",
    //     dining_area_id: 1,
    //     email: &'a str,
    //     pub experience_id: u32,
    //     pub experience_version: u32,
    //     pub fbp: &'a str,
    //     pub first_name: &'a str,
    //     pub last_name: &'a str,
    //     pub party_size: u32,
    //     pub points: u32,
    //     pub points_type: &'a str,
    //     pub reservation_attribute: &'a str,
    //     pub reservation_date_time: &'a time::PrimitiveDateTime,
    //     pub reservation_type: &'a str,
    //     pub restaurant_id: u32,
    //     pub slot_availability_token: &'a str,
    //     pub slot_hash: u32,
    //     pub slot_lock_id: u32,
    //     pub phone_number: &'a str
    // };

    Ok(())
}
