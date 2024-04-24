use std::{collections::btree_set::SymmetricDifference, io::{self, stdout}};
use tablehog::*;
use serde_json::value::Value;

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    let client = reqwest::Client::new();

    let csrf_token = obtain_csrf_token(&client).await?;
    println!("csrf_token: {}", csrf_token);

    let restaurant_id = 118267; // Maude
    let experience_id = 285395; // Maude 10th Anniversary
    let party_size = 2;
    let date = time::macros::date!(2024-05-01);
    let time_ = time::macros::time!(18:30);
    let forward_minutes = 180;
    let backwards_minutes= 0;
    let forward_days = 30;

    let response = fetch_experience_availability(
        &client, 
        restaurant_id, 
        experience_id, 
        party_size, 
        &date, 
        &time_, 
        forward_minutes, 
        backwards_minutes, 
        forward_days,
        &csrf_token
    ).await?;

    println!("response: \n{:#?}", response);

    let response_json = response.json::<Value>().await?;

    println!("response: {}", response_json);

    Ok(())
}






// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let client = Client::new();

//     let response = client.post("https://www.opentable.com/dapi/fe/gql?optype=query&opname=ExperienceAvailability")
//         .header("accept", "*/*")
//         .header("accept-language", "en-US,en;q=0.9")
//         .header("content-type", "application/json")
//         .header("cookie", "otuvid=E5D2BA16-2442-4BDA-8726-DE0010AE5491; _gcl_au=1.1.2069233118.1712703978; _fbp=fb.1.1712703978376.7255612793532183; otuvid_f=59dca7d7-b8f2-4dee-8a92-f77ed939db07; otuvid_p=9e94cedc-bce3-42d8-b99e-1069c5b8cbd1; otuvid_t=e99aacb2-1e66-4351-b0bb-29ef284dd014; ha_userSession=lastModified=2024-04-18T17%3A03%3A14.000Z&origin=prod-sc; _gid=GA1.2.333556256.1713571353; ak_bmsc=AD66A4D086989C5E3C3B7882119E3C37~000000000000000000000000000000~YAAQD/fereOaRQKPAQAAQ8uDAhd2cRSQejitOZ6ER8Zq5bRPQR+XmqAAkXbuQIGXcW3cMSTDboZVMPhAVZMBiyLEYdbaTjcYXbX5oRv2PrydwxEgysj8mPBPiYYA3Iv2gYdrQYQiM+JnsIdkslxfph/wAaXNcw/gCyk/gI5+gi8s0u1i+k2DLZHZW42Jxqw7f1Z4k/67Arbkmgg1u8dm6x6Zc18Rd+rOW8WLREXo/1CocLKxLJofQyTY9/fwF4Bq+VG5julGEAhI4FDVxEUNKsD/XfGB0XqLlBcRp02obyTn6QvTqhUYnA8rFwTpF7zRSuyLug5nkcpluZQarij1HSkuVLyrQ12juSbP+zxVGsEEusRrcKQykU/9SnIgZR/LkXITe4pVk4x77Kot+HhPPKAD/FVIbUcaQZp2ItQAvSPcQEO7c62Qr0iCS+JC6RcIS4TPFsN6ZB7rn9xM; _gat_UA-52354388-1=1; OT-SessionId=fe1b39d8-e0b5-4ba7-92a5-2bcf9845cd4c; _dc_gtm_UA-52354388-1=1; OT_dtp_values=datetime=2024-04-21 17:00; _ga=GA1.2.536058515.1712703978; OptanonConsent=isGpcEnabled=0&datestamp=Sun+Apr+21+2024+15%3A04%3A06+GMT-0700+(Pacific+Daylight+Time)&version=202402.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=3d16055f-9fd6-4110-90d7-6e97aa5e4e3e&interactionCount=1&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A1%2CC0003%3A1%2CC0004%3A1&AwaitingReconsent=false; _uetsid=4a0923b0fea911ee87081d4213d27953; _uetvid=c6299ef0f6c511ee89de892746a54497; _ga_Y77FR7F6XF=GS1.1.1713737001.11.1.1713737051.0.0.0; ftc=x=2024-04-21T23%3A04%3A11&c=1&pt1=1&pt2=1&er=118267&p1ca=restaurant%2Fprofile%2F118267&p1q=corrid%3D321ab424-4288-4f4a-9354-5d0994b9015b%26p%3D2%26sd%3D2024-04-19T19%3A00%3A00; OT-Session-Update-Date=1713737051; bm_sv=6AD05A06819BD5A5ABA62E26CDE7ED4A~YAAQD/ferSkYUAKPAQAAhy+wAhd2VaqF55BFcQWMzo/AVFkm9R4wyRPNlnw54BxbVp134SVWe1/p7UNyX98uFerf6WdQKCO+3QcMrtKJRdsXw2NhA0AAk/XSfSd57t/JQGjPkh2gix6UZ+h/moLdNAILvyx7XYH85Xe0pnzIEUWJCTijTjfk0yQ3LHt9Sgkn9wGF+v+Py4uLhqx6hOXMd/+c7cRTu5+bzZHkl3ZO8oMSjNybvZzgC9ne34AEoFC2vzjh+Q==~1")
//         .header("origin", "https://www.opentable.com")
//         .header("ot-page-group", "booking")
//         .header("ot-page-type", "experiences_availability")
//         .header("priority", "u=1, i")
//         .header("referer", "https://www.opentable.com/booking/experiences-availability?rid=118267&experienceId=285395&modal=true&covers=2&dateTime=2024-04-21T17:00:00")
//         .header("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"")
//         .header("sec-ch-ua-mobile", "?0")
//         .header("sec-ch-ua-platform", "\"macOS\"")
//         .header("sec-fetch-dest", "empty")
//         .header("sec-fetch-mode", "cors")
//         .header("sec-fetch-site", "same-origin")
//         .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
//         .header("x-csrf-token", "fe4b0a35-5b3d-4a80-9bee-4a6714c63021")
//         .header("x-query-timeout", "6883")
//         .body(r#"{"operationName":"ExperienceAvailability","variables":{"includeDiningAreas":true,"transformOutdoorToDefault":false,"restaurantIds":[118267],"partySize":2,"dateTime":"2024-05-07T18:30","experienceId":285395,"type":"Experience","returnTimeSlots":true,"backwardMinutes":45,"forwardMinutes":150,"forwardDays":30},"extensions":{"persistedQuery":{"version":1,"sha256Hash":"9a7cd200454543087f0c500e9ac7fd04a811a107c0f8e1eca7f2714cbfeaf4e0"}}}"#)
//         .send()
//         .await?;

//     println!("Response: {:?}", response.text().await?);

//     Ok(())
// }