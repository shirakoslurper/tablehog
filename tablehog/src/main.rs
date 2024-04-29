use anyhow::Context;
use std::ops::Sub;
use tablehog::*;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    

    // let cc_dt_format = time::format_description::parse("[day]/[month]/[year]")?;
    // let dt = time::Date::parse("01/05/2024", &cc_dt_format)?;
    // let mmyy_format = time::format_description::parse("[month][year repr:last_two]")?;
    // let mmyy_dt_str = dt.format(&mmyy_format)?;

    // println!("mmyy: {}", mmyy_dt_str);

    // let current_offset_date_time = unix_offset_date_time_now_local()?;
    // println!(
    //     "{}, year: {}, month: {}", 
    //     current_offset_date_time, 
    //     current_offset_date_time.year(), 
    //     current_offset_date_time.month() as u8
    // );

    // // sleep until given time!
    // let task_date_time = time::OffsetDateTime::new_in_offset(
    //     time::macros::date!(2024-04-29),
    //     time::macros::time!(13:01), 
    //     current_offset_date_time.offset()
    // );

    // // Current offset_date_time may have aged a bit but lets rely on computers being fast
    // let sleep_duration = task_date_time.sub(current_offset_date_time);
    // tokio::time::sleep(sleep_duration.unsigned_abs()).await;
    // println!("slept til {}", unix_offset_date_time_now_local()?);    


    // let client = reqwest::Client::new();

    // // let csrf_token = obtain_csrf_token(&client).await?;
    // // println!("csrf_token: {}", csrf_token);

    // // let restaurant_id = 118267; // Maude
    // // let experience_id = 285395; // Maude 10th Anniversary

    // // BOOKER DETAILS
    // let first_name = "Jack";
    // let last_name = "Baxter";
    // let email = "jacquesspeedbaxter@gmail.com";
    // let phone_number = "2133678834";

    // // RESTAURANT & EXPERIENCE DETAILS
    // let restaurant_id = 14410;  // Osteria Mozza
    // let experience_id = 208989; // Pasta Tasting Menu
    // let experience_version = 9;
    // let party_size = 2;
    // let reference_date_time = time::OffsetDateTime::new_in_offset(
    //     time::macros::date!(2024-05-06),
    //     time::macros::time!(18:00), 
    //     current_offset_date_time.offset()
    // );
    // let forward_minutes = 240;
    // let backwards_minutes= 0;
    // let forward_days = 30;

    // // DIFFICULT TO FETCH DETAILS
    // let fbp = "fb.1.1714196411819.1399697290534310";

    // let fetch_experience_availability_response = fetch_experience_availability(
    //     &client, 
    //     restaurant_id, 
    //     experience_id, 
    //     party_size, 
    //     &reference_date_time, 
    //     forward_minutes, 
    //     backwards_minutes, 
    //     forward_days,
    // ).await?;

    // println!("fetch_experience_availability_response: \n{:#?}", fetch_experience_availability_response);

    // let deser_fetch_experience_availability_response = fetch_experience_availability_response.json::<FetchExperienceAvailabilityResponse>().await?;

    // println!("deser_fetch_experience_availability_response: {:#?}", deser_fetch_experience_availability_response);

    // let day_offset_to_experience_slots = available_experience_slots(&deser_fetch_experience_availability_response);

    // println!("day_offset_to_experience_slots: {:#?}", day_offset_to_experience_slots);

    // let locked_slot = lock_first_available_slot(
    //     &client, 
    //     &day_offset_to_experience_slots, 
    //     &LockFirstAvailableSlotDetails {
    //         restaurant_id,
    //         reference_date_time,
    //         party_size,
    //         experience_id,
    //         experience_version
    //     }
    // )
    // .await?
    // .context("Failed to lock a slot")?;

    // println!("locked_slot: {:#?}", locked_slot);

    // // PLACEHOLDER CARD DETAILS
    // // TODO: HAVE DATE DETAILS BE IN OFFSETDATETIME
    // // SO WE CAN CHOOSE TO FORMAT IT DIFFERENTLY
    // // EX: MMYY OR AS BELOW
    // let card_details = CardDetails {
    //     number: "4347697073379635",
    //     cvv: "635",
    //     first_name: "Jack",
    //     last_name: "Baxter",
    //     month: 5,
    //     year: 2027,
    //     zip_code: "90025"
    // };
    // // TODO: PARSE CREDIT_CARD_LAST FOUR
    // let credit_card_last_four = "9635";
    // let credit_card_mmyy = "0527";

    // println!("adding card to spreedly");

    // let spreedly_add_payment_method_response = spreedly_add_payment_method(
    //     &client, 
    //     &card_details
    // ).await?;

    // println!("spreedly_add_payment_method_response: {:#?}", spreedly_add_payment_method_response);

    // let deser_spreedly_add_payment_method_response = spreedly_add_payment_method_response.json::<SpreedlyAddPaymentMethodResponse>().await?;

    // println!("deser_spreedly_add_payment_method_response: {:#?}", deser_spreedly_add_payment_method_response);

    // // TODO: CHECK FOR CARD ADDED SUCCESS IN REAL FLOWS
    // if !deser_spreedly_add_payment_method_response.transaction.succeeded {
    //     return Err(anyhow::anyhow!("Failed to add payment method!"));
    // }

    // let make_experience_reservation_details = MakeExperienceReservationDetails{
    //     credit_card_last_four,
    //     credit_card_mmyy,
    //     credit_card_token: &deser_spreedly_add_payment_method_response.transaction.payment_method.token,
    //     dining_area_id: 1,
    //     email,
    //     experience_id,
    //     experience_version,
    //     fbp,
    //     first_name,
    //     last_name,
    //     party_size,
    //     points: locked_slot.experience_slot.points_value,
    //     points_type: &locked_slot.experience_slot.points_type,
    //     reservation_attribute: &locked_slot.chosen_attribute,
    //     reservation_date_time: &locked_slot.date_time,
    //     restaurant_id,
    //     slot_availability_token: &locked_slot.experience_slot.slot_availability_token,
    //     slot_hash: locked_slot.experience_slot.slot_hash,
    //     slot_lock_id: locked_slot.slot_lock_id,
    //     phone_number
    // };

    // let make_experience_reservation_response = make_experience_reservation(
    //     &client,
    //     &make_experience_reservation_details
    // )
    // .await?;

    // println!("make_experience_reservation_response: {:#?}", make_experience_reservation_response);

    // let make_experience_reservation_response_json = make_experience_reservation_response.json::<serde_json::Value>().await?;

    // println!("make_experience_reservation_response_json: {:#?}", make_experience_reservation_response_json);

    Ok(())
}
