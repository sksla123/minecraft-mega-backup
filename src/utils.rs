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
        // 올바른 Public 경로 사용
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
        anyhow::bail!("{} 실행 실패: {}", cmd, err_msg);
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
    let lower_output = df_output.to_lowercase();
    if let Some(idx) = lower_output.find("free:") {
        let rest = &lower_output[idx + 5..].trim_start();
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 2 {
            let val: f64 = parts[0].parse()?;
            let mult = match parts[1] {
                "kb" => 1024.0,
                "mb" => 1024.0 * 1024.0,
                "gb" => 1024.0 * 1024.0 * 1024.0,
                "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                _ => 1.0,
            };
            return Ok((val * mult) as u64);
        }
    }
    anyhow::bail!("mega-df 출력을 파싱할 수 없습니다: {}", df_output)
}

pub fn run_rcon_command(config: &Config, rcon_cmd: &str) -> anyhow::Result<()> {
    if let Some(rcon_cfg) = &config.rcon {
        if rcon_cfg.enable {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // 마인크래프트 RCON 프로토콜 호환성을 위한 quirks 활성화
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
