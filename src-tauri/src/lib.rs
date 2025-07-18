use std::path::PathBuf;
use biliup::uploader::credential::{login_by_cookies, Credential as BiliCredential};
use futures::Stream;
use std::fmt::Write;
use std::pin::Pin;
use std::task::Poll;
use bytes::{Buf, Bytes};
use reqwest::Body;
use tokio::sync::mpsc::UnboundedSender;
use std::sync::{Arc, RwLock};
use biliup::uploader::bilibili::BiliBili;
use tauri::Manager;

pub mod error;

/// B站登录凭证管理结构体
#[derive(Default)]
pub struct Credential {
    pub credential: RwLock<Option<Arc<BiliBili>>>,
}

impl Credential {
    /// 获取当前用户凭证，若无则自动登录
    pub async fn get_current_user_credential(&self, app: &tauri::AppHandle) -> error::Result<Arc<BiliBili>> {
        {
            let read_guard = self.credential.read().unwrap();
            if let Some(cred) = read_guard.as_ref() {
                return Ok(cred.clone());
            }
        }
        let login_info = login_by_cookies(cookie_file(app)?, None).await?;
        let myinfo: serde_json::Value = login_info
            .client
            .get("https://api.bilibili.com/x/space/myinfo")
            .send()
            .await?
            .json()
            .await?;
        let user = config_path(app)?.join(format!("users/{}.json", myinfo["data"]["mid"]));
        user_path(app, user).await?;
        let arc = Arc::new(login_info);
        *self.credential.write().unwrap() = Some(arc.clone());
        Ok(arc)
    }

    /// 清除当前凭证
    pub fn clear(&self) {
        *self.credential.write().unwrap() = None;
    }
}

/// 获取配置文件路径
pub fn config_file(app: &tauri::AppHandle) -> error::Result<PathBuf> {
    Ok(config_path(app)?.join("config.yaml"))
}

/// 获取cookie文件路径
pub fn cookie_file(app: &tauri::AppHandle) -> error::Result<PathBuf> {
    Ok(config_path(app)?.join("cookies.json"))
}

/// 获取配置目录路径，自动创建目录
pub fn config_path(app: &tauri::AppHandle) -> error::Result<PathBuf> {
    let mut config_dir: PathBuf = app.path().config_dir().unwrap();
    config_dir.push("biliup");
    if !config_dir.exists() {
        std::fs::create_dir(&config_dir)?;
    }
    println!("config_path: {config_dir:?}");
    Ok(config_dir)
}

/// 保存用户cookie到指定路径
pub async fn user_path(app: &tauri::AppHandle, path: PathBuf) -> error::Result<PathBuf> {
    let mut users = config_path(app)?;
    users.push("users");
    if !users.exists() {
        std::fs::create_dir(&users)?;
    }
    std::fs::copy(cookie_file(app)?, &path)?;
    println!("user_path: {path:?}");
    Ok(users)
}

/// 通过账号密码登录并保存cookie
pub async fn login_by_password(app: &tauri::AppHandle, username: &str, password: &str) -> anyhow::Result<()> {
    let info = BiliCredential::default().login_by_password(username, password).await?;
    let file = std::fs::File::create(cookie_file(app)?)?;
    serde_json::to_writer_pretty(&file, &info)?;
    println!("密码登录成功，数据保存在{:?}", file);
    Ok(())
}

/// u16数组转十六进制字符串
pub fn encode_hex(bytes: &[u16]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:x}", b).unwrap();
    }
    s
}

/// 上传进度条结构体
#[derive(Clone)]
pub struct Progressbar {
    bytes: Bytes,
    tx: UnboundedSender<u64>,
}

impl Progressbar {
    /// 创建进度条
    pub fn new(bytes: Bytes, tx: UnboundedSender<u64>) -> Self {
        Self { bytes, tx }
    }

    /// 发送进度并返回分片
    pub fn progress(&mut self) -> error::Result<Option<Bytes>> {
        let pb = &self.tx;
        let content_bytes = &mut self.bytes;
        let n = content_bytes.remaining();
        let pc = 1048576; // 每次分片1MB
        if n == 0 {
            Ok(None)
        } else if n < pc {
            pb.send(n as u64)?;
            Ok(Some(content_bytes.copy_to_bytes(n)))
        } else {
            pb.send(pc as u64)?;
            Ok(Some(content_bytes.copy_to_bytes(pc)))
        }
    }
}

impl Stream for Progressbar {
    type Item = error::Result<Bytes>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.progress()? {
            None => Poll::Ready(None),
            Some(s) => Poll::Ready(Some(Ok(s))),
        }
    }
}

impl From<Progressbar> for Body {
    fn from(async_stream: Progressbar) -> Self {
        Body::wrap_stream(async_stream)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_hex() {
        // 测试encode_hex函数
        let data = [0x41u16, 0x42u16];
        assert_eq!(super::encode_hex(&data), "4142");
    }
}
