use indoc::formatdoc;
use reqwest::{Client, Response};
use scraper::{Html, Selector};

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

    let cookie = "otuvid=E5D2BA16-2442-4BDA-8726-DE0010AE5491; _gcl_au=1.1.2069233118.1712703978; _fbp=fb.1.1712703978376.7255612793532183; otuvid_f=59dca7d7-b8f2-4dee-8a92-f77ed939db07; otuvid_p=9e94cedc-bce3-42d8-b99e-1069c5b8cbd1; otuvid_t=e99aacb2-1e66-4351-b0bb-29ef284dd014; _gid=GA1.2.333556256.1713571353; OT_dtp_values=datetime=2024-04-21 17:00; ftc=x=2024-04-22T17%3A38%3A28&c=1&pt1=1&pt2=1; OT-SessionId=0c41e95e-d2f0-48f1-85dd-f3f8239a0c20; OptanonConsent=isGpcEnabled=0&datestamp=Mon+Apr+22+2024+09%3A38%3A30+GMT-0700+(Pacific+Daylight+Time)&version=202402.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=3d16055f-9fd6-4110-90d7-6e97aa5e4e3e&interactionCount=1&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A1%2CC0003%3A1%2CC0004%3A1&AwaitingReconsent=false; _dc_gtm_UA-52354388-1=1; _gat_UA-52354388-1=1; _ga=GA1.1.536058515.1712703978; ak_bmsc=58541929927206F0B3BB29523EDE84FE~000000000000000000000000000000~YAAQ69fOF/ztaPOOAQAAU2KsBhf7LexcmKLKS5Uw5TPuKjt7Sjx3Ne8T0je1b7ZMfeqRSedXZPpByCzGhWl1mf3GMwHCrp6qCvkc1EOThJKo20rvrx0aTBjXun27+iDwdE1N2c+RqPT0W0zAlSwxNtcG0XSgJzNVtAJhZrGL5pam/psuEw90C4KsnUdPBhrhbRruutA6ifCTe5UyBpwMBzbrPaeRSeyD4IGybTVyDR+D7Vn5OmReXsNwXaef6wm1O0HR0CTuzkKXweSD7YmEHLzgc7ZNDxURuUbPn2XIaU4Z/tqM0gTpO/tdlbKEtqvvtc9b4T8YoWg7RIH2BdF8tUUPPLIw49T03950grs2B99o9jFneEPK8cv9UhZqAEK9B5NuwYNSLRaxB2yZEPoHjFzs8Op9ytyVphsC3DGF+oRiGipuSN3wUhD0N8ZjOdkY4IaAJFlMR1D+F/uL; _uetsid=4a0923b0fea911ee87081d4213d27953; _uetvid=c6299ef0f6c511ee89de892746a54497; OT-Session-Update-Date=1713803912; bm_sv=157FC119B1590007B8F562AE6D4DABAC~YAAQ69fOF6fvaPOOAQAAQGesBhe+Z3hSW/Ie0eMiwhpZI50k29FXUP+gD5gV2P+SoTkwKIHzCjJC76yKBJcxjmgcyAUlldMXGQmLpOgZJyIZMqbIc73XnCMF9H57TGKKzrMoti/FTrJo0tEiZctcrYDfjTbU9CnF8kobFh1yFYzWefJ241lTdY1rcFu6viiZBklQdzvjTU2JyW9YtzdhThMQs5AQm56FktnYLu3m2l2QF0vHPEloetnIM4RR4fPfhgkn~1; _ga_Y77FR7F6XF=GS1.1.1713803900.14.1.1713803915.0.0.0";

    println!("obtaining CSRF token");
    let html = client.get(OPENTABLE_URL)
        .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("accept_language", "en-US,en;q=0.9")
        .header("cookie", cookie)
        .header("priority", "u=0, i")
        .header("referer", "https://www.google.com/")
        .header("sec-ch-ua", r#""Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .header("sec-fetch-dest", "document")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-site", "cross-site")
        .header("sec-fetch-user", "?1")
        .header("upgrade-insecure-requests", "1")
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .send()
        .await?
        .text()
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

    let cookie = "otuvid=E5D2BA16-2442-4BDA-8726-DE0010AE5491; _gcl_au=1.1.2069233118.1712703978; _fbp=fb.1.1712703978376.7255612793532183; otuvid_f=59dca7d7-b8f2-4dee-8a92-f77ed939db07; otuvid_p=9e94cedc-bce3-42d8-b99e-1069c5b8cbd1; otuvid_t=e99aacb2-1e66-4351-b0bb-29ef284dd014; ha_userSession=lastModified=2024-04-18T17%3A03%3A14.000Z&origin=prod-sc; _gid=GA1.2.333556256.1713571353; OT_dtp_values=datetime=2024-04-21 17:00; OT-SessionId=2d2a06d5-a4b5-4405-bb8e-b468cf4e0e64; _ga=GA1.2.536058515.1712703978; _dc_gtm_UA-52354388-1=1; _gat_UA-52354388-1=1; _uetsid=4a0923b0fea911ee87081d4213d27953; _uetvid=c6299ef0f6c511ee89de892746a54497; _ga_Y77FR7F6XF=GS1.1.1713746243.12.1.1713746249.0.0.0; ftc=x=2024-04-22T01%3A37%3A29&px=1&c=1&pt1=1&pt2=1&er=118267&p1ca=restaurant%2Fprofile%2F118267&p1q=corrid%3D321ab424-4288-4f4a-9354-5d0994b9015b%26p%3D2%26sd%3D2024-04-19T19%3A00%3A00; OT-Session-Update-Date=1713746249; bm_mi=D73127449117CE625F1F3974C379D91F~YAAQFPferV/LxeuOAQAAwIg8AxeXuvgfGxVJ0BFPoeyr4JgGJzXYsd3N2wODMgGCM3EkZY/JLGiUNh23qkjGAWyOXComSAcErlv018pyQaXEw4epFTTygynS70nBKc5a+nafI5YpmyOzRaQ2xk9g1jdVps7bQvsqKdyHtiVZSPvFhxAVLoGdSJF4PCD2w34pzpaBdKn3ATQK40dWY1QOA+CfWnCWanz4g7X5jAowh9KeHem8zAudMM+EzpoIhNM+vZ6qDsUlxjZlediyrNUtMTpoW/tuRDCnlZM2s2EMd2KYMU7tHvHWbdfDO8yGm5yRFd1EP6J359IAMh9R1P/cYxSykZS0K4A3XQVCrrZX9aoj~1; bm_sv=8E4D9AC69E69CE757B4663F5B6F75D02~YAAQFPferWDLxeuOAQAAwIg8AxcWlyLwaj/9IPhwltbou4piHwHTzB4oSpv7xhtDHcjbRd0/VOvTc+GAUKbthWpShyIyKTUOhFkn/wAhkLobHVToCI/RfDVLg6tK/1CubKLOy9AWuYGRuEK3Eq+VbhohM0gRLuC2uJqUiaKh4cK2NLivBCt4gWuc+nFENZ0XZa3SQRZj0HIMtw7ZiDc3AsDB2q6wqjByTmm2ZGthCCr5R/sdyOcbZz/wTb9u6ky7Zo3x~1; ak_bmsc=F2DC2B87A526AE6DE28314E50F2A0CD9~000000000000000000000000000000~YAAQFPferXLLxeuOAQAAMok8Axf8BdK1TOMD89qJUYYm51h9CpHNCJMMnLAJahUzHJ4IY8XeERv72cWNP1nCIDZvsYaqwEYuUYoUUixphTMBZ1nydR9ibDsjdgCgxwFx5ucW8fYpmCTRV5HSFsF/HiDhbpzjZZIBTx/AqDOq+rYuCSycfyGF5jlxqKhzfu2XvK0Yy/VTCm9q5mt66ImiL/AXS0VlIIiFg5/0XlfBIPLkPx+NVDmWGiEdkJ00iI7sMOJFEC7K+CFJbKvIu5CabWwOVbDBiYGiwvbwkek5zXnvTHa0XPdKBF6Lq8k4eeM4psejDPEBDiRTswhnd+g+sWhDYlVhrXtHu0KcZq2xxZvFn5zBTuzm9Kb88+AVBS7mw69Qlt3ija+sUWrzGRgedpyy2YbOVcb/OhxPnYEOpXpxnXLCJKAaaqHdAV85NAluluDm6YH1CeY3WNaU+ydk1rwGPlkRE+L+Zl8TwblF8CW3fvwoTyglT9JLh6q7ZJXVzIVKwYiM; OptanonConsent=isGpcEnabled=0&datestamp=Sun+Apr+21+2024+17%3A37%3A30+GMT-0700+(Pacific+Daylight+Time)&version=202402.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&consentId=3d16055f-9fd6-4110-90d7-6e97aa5e4e3e&interactionCount=1&isAnonUser=1&landingPath=NotLandingPage&groups=C0001%3A1%2CC0002%3A1%2CC0003%3A1%2CC0004%3A1&AwaitingReconsent=false";
    let x_csrf_token = "15243d13-21d9-474f-83b6-45f744e519b4";
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
        .header("cookie", cookie)
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
        .header("x-csrf-token", x_csrf_token)
        .header("x-query-timeout", "6883")
        .body(body)
        .send()
        .await
        .map_err(|e| e.into())
}