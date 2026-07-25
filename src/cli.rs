use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Minecraft-mega backup (mcmgb)",
    version = "0.1.0",
    about = "MEGA 저장소를 이용한 마인크래프트 서버 데몬 백업 툴",
    long_about = None
)]
pub struct Cli {
    /// 설정 파일 경로 지정
    #[arg(short, long, default_value = "config.toml", help = "설정 파일의 경로를 지정합니다.")]
    pub config: String,

    /// 설정 파일 및 시스템 연동 상태 종합 점검
    #[arg(long, help = "의존성, 마인크래프트 연동, MEGA 연동, 알림(Shoutrrr) 상태를 점검합니다.")]
    pub check: bool,

    /// 데몬 시작 시 연동 실패(예: 서버 부팅 중) 시 재시도 횟수
    #[arg(long, help = "시작 시점의 초기 연동 재시도 횟수를 지정합니다.")]
    pub retry: Option<u32>,

    /// 재시도 간격 (예: 10s, 1m)
    #[arg(long, help = "재시도 간 대기 시간을 지정합니다 (예: 30s, 1m).")]
    pub retry_interval: Option<String>,
}

