use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use serde_json::{json, Value};
use signal_hook::flag;
use telegram_bot_api::{bot, methods::SendMessage, types::ChatId};
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let term = Arc::new(AtomicBool::new(false));
    let sched = JobScheduler::new().await?;

    let job = Job::new_async(env::var("CRON").unwrap().as_str(), |_uuid, _l| {
        Box::pin(async move {
            let client = reqwest::Client::new();

            let name = env::var("NAME").unwrap();
            let id = env::var("ID").unwrap();

            let res = client
                .post("http://82.157.138.16:8091/CRAC/app/businessSupport/cracOperationCert/getOperCertByParamWeb")
                .header("Content-Type", "application/json")
                .body(
                    json!(
                        {
                            "req": {
                            "page_no": "1",
                            "page_size": "100",
                            "name": name,
                            "certificateNo": "",
                            "idCarNumber": id
                            }
                          }
                    )
                    .to_string(),
                )
                .send().await;

            match res {
                Ok(rs) => {
                    let rs = &rs.text().await.unwrap();
                    let data: Value = serde_json::from_str(rs).unwrap();
                    if json!(10000) != *data.get("code").unwrap() {
                        println!("{}", data.get("msg").unwrap());
                        return;
                    }

                    let mut message = String::new();
                    let prc_list = data["res"]["prcList"].as_array().unwrap();
                    let mut it = prc_list.iter().peekable();
                    while let Some(p) = it.next() {
                        message.push_str(&format!(
                            "{}类, 编号：{}, 颁发日期：{};",
                            p["type"].as_str().unwrap(),
                            p["certificateNo"].as_str().unwrap(),
                            p["issueDate"].as_str().unwrap()
                        ));
                        if it.peek().is_some() {
                            message.push('\n');
                        }
                    }

                    let bot_token = env::var("BOT_TOKEN").unwrap();
                    let chat_id = env::var("CHAT_ID").unwrap();
                    let bot = bot::BotApi::new(bot_token, None).await.unwrap();
                    let req = SendMessage::new(ChatId::StringType(chat_id), message);

                    let res = bot.send_message(req).await;
                    if res.is_err() {
                        println!("Telegram bot API failed!");
                    }
                }
                _ => {
                    println!("Cert api failed！")
                }
            }
        })
    })?;
    let uuid = sched.add(job).await?;

    println!("added job: {}", uuid);
    sched.start().await?;
    println!("schedular started!");

    flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    while !term.load(Ordering::Relaxed) {
        // Wait here
        tokio::time::sleep(core::time::Duration::from_secs(1)).await;
    }
    println!("shutdown");
    Ok(())
}
