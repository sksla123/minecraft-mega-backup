// src/backup.rs
use std::fs::{self, File};
use std::path::Path;
use std::thread;
use std::time::Duration;
use chrono::Local;
use crate::config::Config;
use crate::utils::{notify, run_mega_cmd, get_dir_size, parse_mega_free_space, run_rcon_command};

pub fn run(config: &Config) -> anyhow::Result<()> {
    let remote_dir = &config.mega.remote_dir;
    let shoutrrr_url = &config.shoutrrr.url;

    notify(shoutrrr_url, "🚀 마인크래프트 서버 백업 사이클을 시작합니다.");

    let _ = run_mega_cmd("mega-mkdir", &["-p", remote_dir]);

    let ls_output = run_mega_cmd("mega-ls", &[remote_dir])?;
    let backup_folders: Vec<String> = ls_output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    notify(shoutrrr_url, "[STEP 0] 잔존 백업 검사 및 클린업을 시작합니다.");
    let mut valid_backups = Vec::new();

    for folder in &backup_folders {
        let target_path = format!("{}/{}", remote_dir, folder);
        let folder_ls = match run_mega_cmd("mega-ls", &[&target_path]) {
            Ok(out) => out,
            Err(_) => continue,
        };

        if folder_ls.lines().any(|line| line.trim() == "SUCCESS") {
            valid_backups.push(folder.clone());
        } else {
            notify(shoutrrr_url, &format!("🗑️ 비정상 잔존 백업 삭제 중: {}", folder));
            run_mega_cmd("mega-rm", &["-r", &target_path])?;
        }
    }

    notify(shoutrrr_url, "[STEP 1] 마인크래프트 서버 로컬 용량을 계산합니다.");
    let mc_size_bytes = get_dir_size(&config.minecraft.server_dir);
    let mc_size_gb = mc_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    notify(shoutrrr_url, &format!("로컬 서버 용량: {:.2} GB", mc_size_gb));

    notify(shoutrrr_url, "[STEP 2] MEGA 스토리지 잔여 공간을 확인합니다.");
    let df_output = run_mega_cmd("mega-df", &[])?;
    let mega_free_bytes = parse_mega_free_space(&df_output)?;
    
    let extra_margin_bytes = config.mega.extra_margin_gb * 1024 * 1024 * 1024;
    let required_bytes = mc_size_bytes + extra_margin_bytes;

    let mega_free_gb = mega_free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let required_gb = required_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    
    notify(shoutrrr_url, &format!("MEGA 남은 용량: {:.2} GB (필요 용량: {:.2} GB)", mega_free_gb, required_gb));

    if mega_free_bytes < required_bytes {
        anyhow::bail!("MEGA 스토리지 용량 부족. (필요: {:.2} GB, 남은 공간: {:.2} GB)", required_gb, mega_free_gb);
    }

    notify(shoutrrr_url, "[STEP 3] 오래된 백업 로테이션을 점검합니다.");
    valid_backups.sort(); 

    if valid_backups.len() >= config.mega.keep_versions {
        let oldest = &valid_backups[0];
        let oldest_path = format!("{}/{}", remote_dir, oldest);
        let success_file_path = format!("{}/SUCCESS", oldest_path);

        notify(shoutrrr_url, &format!("가장 오래된 백업 삭제 시작: {}", oldest));
        
        run_mega_cmd("mega-rm", &[&success_file_path])?;
        run_mega_cmd("mega-rm", &["-r", &oldest_path])?;
        
        notify(shoutrrr_url, &format!("오래된 백업 삭제 완료: {}", oldest));
    } else {
        notify(shoutrrr_url, "삭제할 오래된 백업이 없습니다 (기준 개수 미달).");
    }

    notify(shoutrrr_url, "[STEP 4] 아카이브 압축 및 백업 업로드를 시작합니다.");
    fs::create_dir_all(&config.minecraft.backup_temp_dir)?;

    if let Err(e) = run_rcon_command(config, "save-off") {
        eprintln!("[WARN] save-off 실행 실패: {}", e);
    }
    if let Err(e) = run_rcon_command(config, "save-all") {
        eprintln!("[WARN] save-all 실행 실패: {}", e);
    }
    thread::sleep(Duration::from_secs(5));

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let local_tar_name = format!("mc_backup_{}.tar.gz", timestamp);
    let local_tar_path = Path::new(&config.minecraft.backup_temp_dir).join(&local_tar_name);

    let tar_gz_file = File::create(&local_tar_path)?;
    let enc = flate2::write::GzEncoder::new(tar_gz_file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(enc);
    let append_result = tar_builder.append_dir_all(".", &config.minecraft.server_dir);
    let finish_result = tar_builder.finish();
    drop(tar_builder);

    if let Err(e) = run_rcon_command(config, "save-on") {
        eprintln!("[WARN] save-on 실행 실패: {}", e);
    }

    append_result?;
    finish_result?;

    let current_remote_dir = format!("{}/{}", remote_dir, timestamp);
    run_mega_cmd("mega-mkdir", &[&current_remote_dir])?;

    let local_tar_str = local_tar_path.to_str().unwrap();
    run_mega_cmd("mega-put", &[local_tar_str, &current_remote_dir])?;

    let _ = fs::remove_file(&local_tar_path);

    notify(shoutrrr_url, "[STEP 5] SUCCESS 마커 파일을 생성합니다.");
    
    let local_success_path = Path::new(&config.minecraft.backup_temp_dir).join("SUCCESS");
    File::create(&local_success_path)?;
    
    run_mega_cmd("mega-put", &[local_success_path.to_str().unwrap(), &current_remote_dir])?;
    let _ = fs::remove_file(&local_success_path);

    notify(shoutrrr_url, "✅ 모든 백업 프로세스가 성공적으로 완료되었습니다.");
    Ok(())
}

