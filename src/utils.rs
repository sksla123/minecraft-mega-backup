use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;
use crate::config::Config;

pub fn parse_interval(interval: &str) -> anyhow::Result<u64> {
    let interval = interval.trim();
    if interval.is_empty() {
        anyhow::bail!("인터벌 문자열이 비어있습니다.");
    }

    let last_char = interval.chars().last().unwrap();
    let num_str = &interval[..interval.len() - 1];
    let num: u64 = num_str.parse().map_err(|_| anyhow::anyhow!("숫자 파싱 실패: {}", num_str))?;

    match last_char {
        'd' | 'D' => Ok(num * 86400),
        'h' | 'H' => Ok(num * 3600),
        'm' | 'M' => Ok(num * 60),
        _ => anyhow::bail!("지원하지 않는 시간 단위입니다 (d, h, m 사용): {}", last_char),
    }
}

pub fn notify(shoutrrr_url: &str, message: &str) {
    println!("[알림] {}", message);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = shoutrrr::transport::ReqwestClient::new();
        if let Err(e) = shoutrrr::send(&client, shoutrrr_url, message).await {
            eprintln!("[WARN] Shoutrrr 알림 발송 실패: {:?}", e);
        }
    });
}

pub fn run_mega_cmd(cmd: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{} 실행 실패:\n```text\n{}\n```", cmd, err_msg);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn get_dir_size<P: AsRef<Path>>(path: P) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|m| m.len())
        .sum()
}

pub fn parse_mega_free_space(df_output: &str) -> anyhow::Result<u64> {
    // 모든 대문자를 소문자로 바꾸고, 띄어쓰기/엔터 구분 없이 단어 단위로 쪼갭니다.
    let lower_output = df_output.to_lowercase();
    let parts: Vec<&str> = lower_output.split_whitespace().collect();

    // 1. "storage:" 글자가 몇 번째 단어인지 찾습니다.
    if let Some(storage_idx) = parts.iter().position(|&s| s == "storage:") {
        // 2. 그 뒤에 "of" 글자가 몇 번째 단어인지 찾습니다.
        if let Some(of_offset) = parts[storage_idx..].iter().position(|&s| s == "of") {
            let of_idx = storage_idx + of_offset;

            // [사용된 용량 (Used) 파싱]
            let used_str = parts.get(storage_idx + 1).unwrap_or(&"0");
            let used_val: f64 = used_str.parse().unwrap_or(0.0);
            let mut used_mult = 1.0;
            
            if let Some(unit_str) = parts.get(storage_idx + 2) {
                // "0.00%" 처럼 퍼센트 기호가 있으면 단위가 생략된 바이트(Byte)로 간주합니다.
                if !unit_str.ends_with('%') {
                    used_mult = match *unit_str {
                        "kb" => 1024.0,
                        "mb" => 1024.0 * 1024.0,
                        "gb" => 1024.0 * 1024.0 * 1024.0,
                        "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                        _ => 1.0,
                    };
                }
            }
            let used_bytes = (used_val * used_mult) as u64;

            // [전체 용량 (Total) 파싱]
            let total_str = parts.get(of_idx + 1).unwrap_or(&"0");
            let total_val: f64 = total_str.parse().unwrap_or(0.0);
            let mut total_mult = 1.0;

            if let Some(unit_str) = parts.get(of_idx + 2) {
                total_mult = match *unit_str {
                    "kb" => 1024.0,
                    "mb" => 1024.0 * 1024.0,
                    "gb" => 1024.0 * 1024.0 * 1024.0,
                    "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                    _ => 1.0,
                };
            }
            let total_bytes = (total_val * total_mult) as u64;

            // 잔여 용량 반환 (전체 - 사용됨)
            if total_bytes >= used_bytes {
                return Ok(total_bytes - used_bytes);
            } else {
                return Ok(0);
            }
        }
    }

    // 구버전(Free: ...) 폴백 로직
    if let Some(idx) = parts.iter().position(|&s| s == "free:") {
        if let Some(val_str) = parts.get(idx + 1) {
            let val: f64 = val_str.parse().unwrap_or(0.0);
            let mut mult = 1.0;
            if let Some(unit_str) = parts.get(idx + 2) {
                mult = match *unit_str {
                    "kb" => 1024.0,
                    "mb" => 1024.0 * 1024.0,
                    "gb" => 1024.0 * 1024.0 * 1024.0,
                    "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                    _ => 1.0,
                };
            }
            return Ok((val * mult) as u64);
        }
    }

    // 파싱 실패 시 디스코드 마크다운이 깨지지 않도록 코드 블록 처리
    anyhow::bail!("mega-df 출력을 파싱할 수 없습니다:\n```text\n{}\n```", df_output)
}

pub fn run_rcon_command(config: &Config, rcon_cmd: &str) -> anyhow::Result<()> {
    if let Some(rcon_cfg) = &config.rcon {
        if rcon_cfg.enable {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut conn = match rcon::Connection::builder()
                    .enable_minecraft_quirks(true)
                    .connect(&rcon_cfg.address, &rcon_cfg.password)
                    .await
                {
                    Ok(c) => c,
                    Err(e) => anyhow::bail!("RCON 서버 접속 실패: {}", e),
                };

                match conn.cmd(rcon_cmd).await {
                    Ok(resp) => {
                        println!("[RCON] 명령어 성공 ({}): {}", rcon_cmd, resp.trim());
                        Ok(())
                    }
                    Err(e) => anyhow::bail!("RCON 명령어 실행 실패: {}", e),
                }
            })?;
        }
    }
    Ok(())
}


