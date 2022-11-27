use tgbot::{Api, UpdateHandler, webhook};
use futures_util::future::BoxFuture;
use std::env;
use tgbot::methods::SendMessage;
use tgbot::types::{Update, UpdateKind};

fn refuel_reply(text: &str) -> Option<String> {
    let mut maybe_km: Option<f64> = None;
    let mut maybe_l: Option<f64> = None;
    let mut maybe_rub: Option<f64> = None;
    let parts = text.split_whitespace().skip(1);
    for part in parts {
        let s = part.to_lowercase();
        if s.ends_with("km") {
            maybe_km = Some(s.replace("km", "").parse::<f64>().unwrap());
        }
        if s.ends_with("l") {
            maybe_l = Some(s.replace("l", "").parse::<f64>().unwrap());
        }
        if s.ends_with("rub") {
            maybe_rub = Some(s.replace("rub", "").parse::<f64>().unwrap());
        }
    }
    let mut res: Vec<String> = vec![];
    match (maybe_km, maybe_l) {
        (Some(km), Some(l)) => {
            res.push(format!("{:.3} l/100km", 100.0 * l / km));
            res.push(format!("{:.3} km/l", km / l));
        }
        _ => {}
    }
    match (maybe_km, maybe_rub) {
        (Some(km), Some(rub)) => {
            res.push(format!("{:.2} rub/km", rub / km));
        }
        _ => {}
    }
    if !res.is_empty() {
        Some(res.join(", "))
    } else {
        None
    }
}

struct Handler {
    api: Api,
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let api = self.api.clone();
        Box::pin(async move {
            if let UpdateKind::Message(message) = update.kind {
                for text in message.get_text() {
                    let data = &text.data;
                    if data.starts_with("/refuel") {
                        for reply in refuel_reply(data) {
                            let method = SendMessage::new(message.get_chat_id(), reply);
                            api.execute(method).await.unwrap();
                        }
                    } else if data.starts_with("/help") {
                        let method = SendMessage::new(message.get_chat_id(), "Type /refuel ???km ???l ???rub to compute consumption.\n For example: /refuel 500km 30l 2000rub");
                        api.execute(method).await.unwrap();
                    }
                }
            }
        })
    }
}


#[tokio::main]
async fn main() {
    let token = env::var("TOKEN").expect("TOKEN is not set");
    let port = env::var("PORT").expect("POST is not set").parse::<u16>().expect("Post must be an integer");
    let api = Api::new(token).expect("Failed to create API");
    webhook::run_server(([127, 0, 0, 1], port), "/fuel_consumption_calculator_bot", Handler { api }).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::refuel_reply;

    #[test]
    fn test_refuel_reply() {
        assert_eq!(refuel_reply("/refuel  1000km   35l   2400rub "), Some(String::from("3.500 l/100km, 28.571 km/l, 2.40 rub/km")));
        assert_eq!(refuel_reply("/refuel  1000KM   35L 2400RUB"), Some(String::from("3.500 l/100km, 28.571 km/l, 2.40 rub/km")));
        assert_eq!(refuel_reply("/refuel  1000KM   35L"), Some(String::from("3.500 l/100km, 28.571 km/l")));
    }
}
