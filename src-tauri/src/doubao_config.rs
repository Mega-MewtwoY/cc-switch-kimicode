//! Read-only integration with the Doubao desktop app.
//!
//! Doubao owns its Chromium profiles and login cookies. CC Switch only reads
//! the active profile and decrypts the minimum web-session cookies in memory;
//! it never writes Doubao files or persists decrypted credentials.

use aes::Aes128;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

type Aes128CbcDec = cbc::Decryptor<Aes128>;

const KEYCHAIN_SERVICE: &str = "Doubao Safe Storage";
const KEYCHAIN_ACCOUNT: &str = "Doubao";
const COOKIE_HOSTS: [&str; 3] = [".doubao.com", "www.doubao.com", ".bytedance.com"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubaoNativeAccount {
    pub profile_name: String,
}

pub(crate) struct DoubaoWebSession {
    pub cookie_header: String,
    pub ms_token: String,
    pub device_id: String,
    pub signed_quota_url: Option<String>,
}

pub fn support_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|home| home.join("Library/Application Support/Doubao"))
        .ok_or_else(|| "无法确定当前用户目录".to_string())
}

pub fn active_profile_name() -> Result<String, String> {
    let root = support_dir()?;
    let local_state = root.join("Local State");
    let profile = std::fs::read_to_string(&local_state)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|doc| {
            doc.pointer("/profile/last_used")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Default".to_string());

    if root.join(&profile).join("Cookies").is_file() {
        return Ok(profile);
    }

    let mut candidates = std::fs::read_dir(&root)
        .map_err(|e| format!("读取豆包数据目录失败: {e}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("Cookies").is_file())
        .filter_map(|entry| {
            let modified = entry
                .path()
                .join("Cookies")
                .metadata()
                .and_then(|meta| meta.modified())
                .ok()?;
            Some((modified, entry.file_name().to_string_lossy().into_owned()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates
        .into_iter()
        .next()
        .map(|(_, name)| name)
        .ok_or_else(|| "未找到豆包桌面端的本地登录 Profile".to_string())
}

pub fn native_account() -> Result<Option<DoubaoNativeAccount>, String> {
    if !doubao_installed() {
        return Ok(None);
    }
    let profile_name = active_profile_name().unwrap_or_else(|_| "Default".to_string());
    Ok(Some(DoubaoNativeAccount { profile_name }))
}

#[cfg(target_os = "macos")]
pub fn doubao_installed() -> bool {
    Path::new("/Applications/Doubao.app").is_dir()
        || dirs::home_dir().is_some_and(|home| home.join("Applications/Doubao.app").is_dir())
        || support_dir().is_ok_and(|root| root.join("installed_versions/Doubao.app").is_dir())
}

#[cfg(not(target_os = "macos"))]
pub fn doubao_installed() -> bool {
    false
}

fn keychain_secret() -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                KEYCHAIN_SERVICE,
                "-a",
                KEYCHAIN_ACCOUNT,
                "-w",
            ])
            .output()
            .map_err(|e| format!("无法读取豆包钥匙串: {e}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if detail.is_empty() {
                "无法读取 Doubao Safe Storage；请允许 CC Switch 访问钥匙串".to_string()
            } else {
                format!("无法读取 Doubao Safe Storage: {detail}")
            });
        }
        let secret = String::from_utf8_lossy(&output.stdout)
            .trim_end_matches(|c| c == '\r' || c == '\n')
            .as_bytes()
            .to_vec();
        if secret.is_empty() {
            return Err("Doubao Safe Storage 内容为空".to_string());
        }
        Ok(secret)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("豆包桌面端原生用量查询目前仅支持 macOS".to_string())
    }
}

fn derive_cookie_key(secret: &[u8]) -> [u8; 16] {
    let mut key = [0u8; 16];
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(secret, b"saltysalt", 1003, &mut key);
    key
}

fn decrypt_cookie(host: &str, encrypted: &[u8], key: &[u8; 16]) -> Result<String, String> {
    if encrypted.len() <= 3 || !matches!(&encrypted[..3], b"v10" | b"v11") {
        return Err("豆包 Cookie 使用了未知的加密格式".to_string());
    }
    let plaintext = Aes128CbcDec::new_from_slices(key, &[b' '; 16])
        .map_err(|_| "初始化豆包 Cookie 解密失败".to_string())?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted[3..])
        .map_err(|_| "豆包 Cookie 解密失败；请重新登录豆包".to_string())?;

    // Chromium cookie DB schema version 24 prefixes SHA-256(host_key).
    let host_hash = Sha256::digest(host.as_bytes());
    let value = if plaintext.starts_with(host_hash.as_slice()) {
        &plaintext[host_hash.len()..]
    } else {
        plaintext.as_slice()
    };
    String::from_utf8(value.to_vec()).map_err(|_| "豆包 Cookie 内容不是有效文本".to_string())
}

fn device_id(root: &Path) -> String {
    std::fs::read_to_string(root.join("Local State"))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|doc| {
            doc.pointer("/aha/device/device_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

fn latest_signed_quota_url(root: &Path) -> Option<String> {
    const MARKER: &str = "start request here: ";
    const QUOTA_PATH: &str = "/alice/commerce/sale/subscription/overview/";
    let log_dir = root.join("sdk_storage/log");
    let mut logs = std::fs::read_dir(log_dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("saman_"))
        .filter_map(|entry| {
            let modified = entry.metadata().and_then(|meta| meta.modified()).ok()?;
            Some((modified, entry.path()))
        })
        .collect::<Vec<_>>();
    logs.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, path) in logs.into_iter().take(8) {
        let text = std::fs::read_to_string(path).ok()?;
        for line in text.lines().rev() {
            let Some(marker) = line.find(MARKER) else {
                continue;
            };
            let marker = marker + MARKER.len();
            let url = line[marker..].trim();
            if url.starts_with("https://www.doubao.com/")
                && url.contains(QUOTA_PATH)
                && url.contains("a_bogus=")
            {
                return Some(url.to_string());
            }
        }
    }
    None
}

pub(crate) fn load_web_session(profile_name: &str) -> Result<DoubaoWebSession, String> {
    let root = support_dir()?;
    let cookie_path = root.join(profile_name).join("Cookies");
    if !cookie_path.is_file() {
        return Err(format!("豆包 Profile {profile_name} 中没有登录 Cookie"));
    }

    let secret = keychain_secret()?;
    let key = derive_cookie_key(&secret);
    let conn = Connection::open_with_flags(&cookie_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("只读打开豆包 Cookie 数据库失败: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT host_key, name, value, encrypted_value
             FROM cookies
             WHERE host_key IN (?1, ?2, ?3)
             ORDER BY CASE host_key
               WHEN '.bytedance.com' THEN 0
               WHEN '.doubao.com' THEN 1
               WHEN 'www.doubao.com' THEN 2
               ELSE 3 END",
        )
        .map_err(|e| format!("读取豆包 Cookie 表失败: {e}"))?;
    let rows = stmt
        .query_map(
            rusqlite::params![COOKIE_HOSTS[0], COOKIE_HOSTS[1], COOKIE_HOSTS[2]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )
        .map_err(|e| format!("查询豆包 Cookie 失败: {e}"))?;

    let mut cookies = BTreeMap::<String, String>::new();
    for row in rows {
        let (host, name, value, encrypted) =
            row.map_err(|e| format!("解析豆包 Cookie 失败: {e}"))?;
        let value = if value.is_empty() {
            match decrypt_cookie(&host, &encrypted, &key) {
                Ok(value) => value,
                Err(_) => continue,
            }
        } else {
            value
        };
        if !value.is_empty() && !name.contains(|c| c == ';' || c == '\r' || c == '\n') {
            cookies.insert(name, value);
        }
    }

    if !cookies.contains_key("sessionid") && !cookies.contains_key("sessionid_ss") {
        return Err("豆包桌面端尚未登录，未找到有效 sessionid".to_string());
    }
    let ms_token = cookies.get("msToken").cloned().unwrap_or_default();
    let cookie_header = cookies
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ");
    Ok(DoubaoWebSession {
        cookie_header,
        ms_token,
        device_id: device_id(&root),
        signed_quota_url: latest_signed_quota_url(&root),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit};

    type Aes128CbcEnc = cbc::Encryptor<Aes128>;

    #[test]
    fn decrypts_chromium_v24_cookie() {
        let secret = b"test-secret";
        let key = derive_cookie_key(secret);
        let host = ".doubao.com";
        let mut plain = Sha256::digest(host.as_bytes()).to_vec();
        plain.extend_from_slice(b"session-value");
        let encrypted = Aes128CbcEnc::new_from_slices(&key, &[b' '; 16])
            .unwrap()
            .encrypt_padded_vec_mut::<Pkcs7>(&plain);
        let mut payload = b"v10".to_vec();
        payload.extend_from_slice(&encrypted);
        assert_eq!(
            decrypt_cookie(host, &payload, &key).unwrap(),
            "session-value"
        );
    }
}
