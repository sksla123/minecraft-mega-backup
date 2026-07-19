// src/main.rs
mod cli;
mod config;
mod utils;
mod check;
mod backup;

use std::fs;
use std::thread;
use std::time::Duration;
use clap::Parser;
use cli::Cli;
use config::Config;

fn main() {
    let cli = Cli::parse();

    let config_content = fs::read_to_string(&cli.config).unwrap_or_else(|e| {
        eprintln!("설정 파일 읽기 실패 ({}): {}", cli.config, e);
        std::process::exit(1);
    });

    let config: Config = toml::from_str(&config_content).unwrap_or_else(|e| {
        eprintln!("설정 파싱 오류: {}", e);
        std::process::exit(1);
    });

    if cli.check {
        check::run(&config);
        return;
    }

    let retries = cli.retry.or(config.daemon.startup_retry).unwrap_or(0);
    let retry_interval_str = cli
        .retry_interval
        .or(config.daemon.startup_retry_interval.clone())
        .unwrap_or_else(|| "1m".to_string());

    let retry_interval_secs = utils::parse_interval(&retry_interval_str).unwrap_or_else(|e| {
        eprintln!("재시도 간격(retry_interval) 파싱 오류: {}", e);
        std::process::exit(1);
    });

    let mut attempt = 0;
    loop {
        match check::verify_startup(&config) {
            Ok(_) => {
                if attempt > 0 {
                    utils::notify(&config.shoutrrr.url, &format!("✅ 초기 연동 완료 ({}회 재시도).", attempt));
                }
                break;
            }
            Err(e) => {
                if attempt >= retries {
                    utils::notify(&config.shoutrrr.url, &format!("❌ 데몬 구동 실패 (초기 연동 재시도 {}회 초과): {}", attempt, e));
                    std::process::exit(1);
                }
                attempt += 1;
                // 재시도 시마다 Shoutrrr 알림 발송
                utils::notify(&config.shoutrrr.url, &format!("⚠️ 초기 연동 대기 중 ({}). {}초 후 재시도합니다... ({}/{})", e, retry_interval_secs, attempt, retries));
                thread::sleep(Duration::from_secs(retry_interval_secs));
            }
        }
    }

    let interval_secs = utils::parse_interval(&config.daemon.interval).unwrap_or_else(|e| {
        eprintln!("주기(interval) 파싱 오류: {}", e);
        std::process::exit(1);
    });

    utils::notify(&config.shoutrrr.url, &format!("🟢 mcmgb 백업 데몬이 성공적으로 시작되었습니다. (주기: {} 초)", interval_secs));

    loop {
        if let Err(err) = backup::run(&config) {
            utils::notify(&config.shoutrrr.url, &format!("❌ 백업 프로세스 중 치명적 오류 발생: {}", err));
        }

        println!("다음 실행까지 대기합니다...");
        thread::sleep(Duration::from_secs(interval_secs));
    }
}
