use std::collections::HashMap;
use std::path::Path;
use futures::StreamExt;
use log::{error, info};
use crate::atmb::ATMBCrawl;
use crate::atmb::model::Mailbox;
use crate::record::Record;
use crate::smarty::{AdditionalInfo, SmartyClientProxy};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use serde::{Deserialize, Serialize};

mod atmb;
mod record;
mod smarty;
mod utils;
mod cli;

fn init_logger() {
    install_tracing();
    color_eyre::install().unwrap();
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();

    // 获取命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    // 根据命令行参数选择运行模式
    let result = if args.len() > 1 {
        match args[1].as_str() {
            "web" => run_web_server().await,
            _ => run_cli().await,
        }
    } else {
        // 默认使用CLI模式
        run_cli().await
    };

    if let Err(e) = result {
        log::error!("Error: {:?}", e);
        std::process::exit(1);
    }
}

async fn run_cli() -> color_eyre::Result<()> {
    // 使用CLI应用获取用户选择的州
    let cli_app = cli::CliApp::new()?;
    let mailboxes = cli_app.run().await?;

    info!("finished fetching, got [{}] mailboxes in total", mailboxes.len());
    info!("begin to inquire mailbox address info...");

    let mailboxes_info = inquire_mailboxes_info(mailboxes).await?;
    // filter out CMRA and addresses
    let records = mailboxes_info.into_iter().filter_map(|(mailbox, info)| {
        if info.is_cmra() {
            None
        } else {
            Some(Record::from_mailbox_and_info(mailbox, info))
        }
    })
        .collect::<Vec<_>>();

    let out_file = "result/mailboxes.csv";
    info!("saving records to [{}]", out_file);
    save_records(records, out_file)
}

async fn inquire_mailboxes_info(mailboxes: Vec<Mailbox>) -> color_eyre::Result<HashMap<Mailbox, AdditionalInfo>> {
    let client = SmartyClientProxy::new()?;

    let total = mailboxes.len();
    let mailboxes_info = futures::stream::iter(mailboxes.into_iter()).enumerate().map(|(idx, mailbox)| {
        let client = &client;
        async move {
            info!("[{}/{}] fetching mailbox address info for [{}]", idx + 1, total, mailbox.name);

            let address = &mailbox.address;
            let additional_info = match client.inquire_address(address.clone()).await {
                Ok(info) => info,
                Err(e) => {
                    error!("cannot inquire address info for [{}]: {:?}", mailbox.name, e);
                    return None;
                }
            };
            Some((mailbox, additional_info))
        }
    })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    Ok(mailboxes_info.into_iter().filter_map(|info| info).collect::<HashMap<_, _>>())
}

/// write result to CSV file
fn save_records(mut records: Vec<Record>, save_path: impl AsRef<Path>) -> color_eyre::Result<()> {
    records.sort_by(|r1, r2| (&r1.cmra, &r1.rdi).cmp(&(&r2.cmra, &r2.rdi)));
    if let Some(parent) = save_path.as_ref().parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut wtr = csv::Writer::from_path(save_path)?;
    for record in &records {
        wtr.serialize(record)?;
    }
    Ok(())
}

// 添加web服务器功能
#[derive(Serialize, Deserialize, Clone)]
struct WebAppState {
    states: Vec<String>,
    selected_states: Vec<String>,
    credentials: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct SetCredentialsRequest {
    credentials: String,
}

#[derive(Serialize, Deserialize)]
struct SelectStatesRequest {
    states: Vec<String>,
}

#[derive(Serialize)]
struct CrawlResult {
    status: String,
    total_processed: usize,  // 总共处理的地址数
    total_non_cmra: usize,   // 非CMRA地址数
    output_path: String,
    results: Vec<Record>,
}

#[get("/api/states")]
async fn get_states(app_data: web::Data<AppData>) -> impl Responder {
    let atmb = ATMBCrawl::new().unwrap();
    match atmb.get_available_states().await {
        Ok(states) => {
            let mut state = app_data.state.lock().unwrap();
            state.states = states.clone();
            HttpResponse::Ok().json(states)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }
}

#[post("/api/credentials")]
async fn set_credentials(req: web::Json<SetCredentialsRequest>, app_data: web::Data<AppData>) -> impl Responder {
    let mut state = app_data.state.lock().unwrap();
    state.credentials = req.credentials.clone();
    std::env::set_var("CREDENTIALS", &state.credentials);
    HttpResponse::Ok().json(state.clone())
}

#[post("/api/select-states")]
async fn select_states(req: web::Json<SelectStatesRequest>, app_data: web::Data<AppData>) -> impl Responder {
    let mut state = app_data.state.lock().unwrap();
    state.selected_states = req.states.clone();
    HttpResponse::Ok().json(state.clone())
}

#[post("/api/start")]
async fn start_crawling(app_data: web::Data<AppData>) -> impl Responder {
    let atmb = ATMBCrawl::new().unwrap();
    let selected_states: Vec<String>;
    
    {
        let mut state = app_data.state.lock().unwrap();
        state.status = "正在爬取数据...".to_string();
        selected_states = state.selected_states.clone();
    }
    
    let result = if selected_states.is_empty() {
        atmb.fetch().await
    } else {
        atmb.fetch_selected_states(&selected_states).await
    };
    
    match result {
        Ok(mailboxes) => {
            let total_processed = mailboxes.len();
            let mut state = app_data.state.lock().unwrap();
            state.status = "正在查询地址信息...".to_string();
            
            // 查询地址信息
            match inquire_mailboxes_info(mailboxes).await {
                Ok(mailboxes_info) => {
                    // 过滤出非CMRA地址
                    let records = mailboxes_info
                        .into_iter()
                        .filter_map(|(mailbox, info)| {
                            if info.is_cmra() {
                                None
                            } else {
                                Some(Record::from_mailbox_and_info(mailbox, info))
                            }
                        })
                        .collect::<Vec<_>>();

                    let total_non_cmra = records.len();
                    let out_file = "result/mailboxes.csv";

                    // 保存记录
                    if let Err(e) = save_records(records.clone(), out_file) {
                        state.status = format!("保存记录失败: {}", e);
                        HttpResponse::InternalServerError().body(state.status.clone())
                    } else {
                        state.status = "完成".to_string();
                        HttpResponse::Ok().json(CrawlResult {
                            status: state.status.clone(),
                            total_processed,
                            total_non_cmra,
                            output_path: out_file.to_string(),
                            results: records,
                        })
                    }
                }
                Err(e) => {
                    state.status = format!("查询地址信息失败: {}", e);
                    HttpResponse::InternalServerError().body(state.status.clone())
                }
            }
        }
        Err(e) => {
            let mut state = app_data.state.lock().unwrap();
            state.status = format!("爬取数据失败: {}", e);
            HttpResponse::InternalServerError().body(state.status.clone())
        }
    }
}

struct AppData {
    state: std::sync::Mutex<WebAppState>,
}

async fn run_web_server() -> color_eyre::Result<()> {
    let app_data = web::Data::new(AppData {
        state: std::sync::Mutex::new(WebAppState {
            states: Vec::new(),
            selected_states: Vec::new(),
            credentials: std::env::var("CREDENTIALS").unwrap_or_default(),
            status: "准备就绪".to_string(),
        }),
    });
    
    println!("Starting web server at http://127.0.0.1:8080");
    println!("CTRL+C to exit");
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(get_states)
            .service(set_credentials)
            .service(select_states) 
            .service(start_crawling)
            .service(Files::new("/", "./web").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    
    Ok(())
}