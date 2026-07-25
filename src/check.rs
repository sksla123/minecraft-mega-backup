// src/check.rs
use std::path::Path;
use std::process::Command;
use crate::config::Config;
use crate::utils::run_mega_cmd;

pub fn run(config: &Config) {
    println!("========================================");
    println!("🔍 mcmgb 시스템 종합 점검 시작");
    println!("========================================\n");

    let mut all_pass = true;

    println!("[1/4] 의존성(MEGAcmd) 패키지 점검...");
    let required_cmds = vec!["mega-ls", "mega-df", "mega-rm", "mega-mkdir", "mega-put", "mega-whoami"];
    for cmd in required_cmds {
        if let Ok(out) = Command::new("which").arg(cmd).output() {
            if out.status.success() {
                println!("  ✅ {} 찾음", cmd);
                continue;
            }
        }
        println!("  ❌ {} 찾을 수 없음 (MEGAcmd 설치 필요)", cmd);
        all_pass = false;
    }

    println!("\n[2/4] 마인크래프트 서버 연동 점검...");
    if Path::new(&config.minecraft.server_dir).exists() {
        println!("  ✅ 서버 디렉토리 존재 확인됨: {}", config.minecraft.server_dir);
    } else {
        println!("  ❌ 서버 디렉토리를 찾을 수 없음: {}", config.minecraft.server_dir);
        all_pass = false;
    }

    if let Some(rcon_cfg) = &config.rcon {
        if rcon_cfg.enable {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let rcon_success = rt.block_on(async {
                match rcon::Connection::builder()
                    .enable_minecraft_quirks(true)
                    .connect(&rcon_cfg.address, &rcon_cfg.password)
                    .await 
                {
                    Ok(mut conn) => {
                        match conn.cmd("list").await {
                            Ok(resp) => {
                                println!("  ✅ RCON 접속 및 명령어 실행 성공 (응답: {})", resp.trim());
                                true
                            }
                            Err(e) => {
                                println!("  ❌ RCON 'list' 명령어 실패: {}", e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ❌ RCON 접속 실패: {}", e);
                        false
                    }
                }
            });
            
            if !rcon_success {
                all_pass = false;
            }
        } else {
            println!("  ℹ️ RCON 설정이 비활성화되어 있어 연동 테스트를 건너뜁니다.");
        }
    }

    println!("\n[3/4] MEGA 계정 연동 점검...");
    match run_mega_cmd("mega-whoami", &[]) {
        Ok(out) => {
            let email = out.trim();
            println!("  ✅ MEGA 로그인 상태 확인됨 (계정: {})", email);
        }
        Err(e) => {
            println!("  ❌ MEGA 로그인 실패 혹은 연결 문제: {}", e);
            all_pass = false;
        }
    }

    println!("\n[4/4] 알림(Shoutrrr) 시스템 점검...");
    let test_message = "✅ [mcmgb] 시스템 종합 점검 테스트 메시지입니다.";
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let shoutrrr_success = rt.block_on(async {
        let client = shoutrrr::transport::ReqwestClient::new();
        match shoutrrr::send(&client, &config.shoutrrr.url, test_message).await {
            Ok(_) => true,
            Err(e) => {
                println!("  ❌ Shoutrrr 알림 발송 실패: {:?}", e);
                false
            }
        }
    });

    if shoutrrr_success {
        println!("  ✅ Shoutrrr 테스트 메시지 발송 성공");
    } else {
        all_pass = false;
    }

    println!("\n========================================");
    if all_pass {
        println!("🎉 모든 점검 항목을 통과했습니다. 백업 데몬을 실행할 준비가 완료되었습니다.");
        std::process::exit(0);
    } else {
        eprintln!("⚠️ 일부 점검 항목에서 문제가 발생했습니다. 설정을 다시 확인해 주십시오.");
        std::process::exit(1);
    }
}

pub fn verify_startup(config: &Config) -> anyhow::Result<()> {
    run_mega_cmd("mega-whoami", &[])
        .map_err(|e| anyhow::anyhow!("MEGA 연동 확인 실패: {}", e))?;

    if let Some(rcon_cfg) = &config.rcon {
        if rcon_cfg.enable {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match rcon::Connection::builder()
                    .enable_minecraft_quirks(true)
                    .connect(&rcon_cfg.address, &rcon_cfg.password)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow::anyhow!("RCON 포트 접속 실패 (마인크래프트 부팅 대기 중일 수 있음): {}", e)),
                }
            })?;
        }
    }
    Ok(())
}
