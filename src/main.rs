use std::{collections::HashMap, time::Duration};

use reqwest::Client;

const ACT_ROUTES: &[(&str, u32)] = &[("51B", 500), ("27", 750), ("E", 1500)];

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Route {
    route: String,
    direction: String,
    destination: String,
    stops: Vec<Stop>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Stop {
    stop_id: u32,
    name: String,
    latitude: f32,
    longitude: f32,
    order: Option<u32>,
    scheduled_time: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Prediction {
    stop_id: u32,
    trip_id: u32,
    vehicle_id: u32,
    route_name: String,
    predicted_delay_in_seconds: i32,
    predicted_departure: String,
    prediction_date_time: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Trip {
    route_id: String,
    direction_id: u32,
    direction: String,
    schedule_type: String,
    headsign: String,
    destination: String,
    destination2: String,
    trip_start_time: String,
    trip_id: u32,
    trip_number: u32,
    trip_number2: u32,
    position_number: u32,
    stop_id: u32,
    stop_description: String,
    passing_time: String,
    stop_number: Option<u32>,
    stop_number2: String,
    place_id: Option<String>,
    stop_longitude: f32,
    stop_latitude: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = "";

    let client = Client::builder()
        .timeout(Duration::from_secs(30)) // Set a 30-second request timeout
        .build()?; // Build the client

    for route in ACT_ROUTES {
        let radius_url = format!(
            "https://api.actransit.org/transit/stops/37.855/-122.254/{}/true/{}?token={}",
            route.1, route.0, token
        );
        let stops: Vec<Stop> = client.get(radius_url).send().await?.json().await?;
        for stop in stops {
            let trips_url = format!(
                "https://api.actransit.org/transit/stops/{}/tripstoday?token={}",
                stop.stop_id, token
            );
            let trips: Vec<Trip> = client.get(trips_url).send().await?.json().await?;
            let trips_by_id: HashMap<u32, Trip> =
                trips
                    .iter()
                    .fold(HashMap::new(), |mut acc: HashMap<u32, Trip>, trip| {
                        acc.insert(trip.trip_id, trip.clone());
                        acc
                    });
            let prediction_url = format!(
                "https://api.actransit.org/transit/stops/{}/predictions?token={}",
                stop.stop_id, token
            );
            let predictions_check = client.get(prediction_url).send().await?;
            if predictions_check.status() != 200 {
                println!("{}: {}: no data", route.0, stop.name);
                continue;
            }
            let predictions: Vec<Prediction> = predictions_check.json().await?;
            for prediction in &predictions {
                if let Some(trip) = trips_by_id.get(&prediction.trip_id) {
                    println!(
                        "{} {}: {}: {}",
                        route.0, trip.direction, stop.name, prediction.predicted_departure,
                    );
                } else {
                    panic!(
                        "Stop {}: No trip info found for Trip ID {}",
                        stop.stop_id, prediction.trip_id
                    );
                }
            }
        }
    }

    Ok(())
}
