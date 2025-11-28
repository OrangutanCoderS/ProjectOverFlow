use crate::rules::SanitizerRules;
use crate::salt::Salt;
use regex::Regex;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fmt;

/// Kind of redaction performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionKind {
    FilePath,
    Username,
    IpAddress,
    MacAddress,
    Hostname,
    Other,
}

/// Single redaction event for audit / logs.
#[derive(Debug, Clone)]
pub struct RedactionEvent {
    pub path: String,
    pub original_preview: String,
    pub replacement_preview: String,
    pub kind: RedactionKind,
}

/// Summary of what happened during sanitization.
#[derive(Debug, Clone)]
pub struct SanitizationReport {
    pub redactions: Vec<RedactionEvent>,
    pub paranoid: bool,
    pub trusted_skipped: bool,
}

impl SanitizationReport {
    pub fn new(paranoid: bool) -> Self {
        SanitizationReport {
            redactions: Vec::new(),
            paranoid,
            trusted_skipped: false,
        }
    }

    pub fn trusted_skip(paranoid: bool) -> Self {
        SanitizationReport {
            redactions: Vec::new(),
            paranoid,
            trusted_skipped: true,
        }
    }

    pub fn total_redactions(&self) -> usize {
        self.redactions.len()
    }
}

impl fmt::Display for SanitizationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SanitizationReport {{ redactions: {}, paranoid: {}, trusted_skipped: {} }}",
            self.total_redactions(),
            self.paranoid,
            self.trusted_skipped
        )
    }
}

/// Main sanitizer object. Cheap to clone, safe to share between threads.
#[derive(Debug, Clone)]
pub struct ExportSanitizer {
    rules: SanitizerRules,
    salt: Salt,
    re_ip: Regex,
    re_mac: Regex,
    re_unix_home: Regex,
    re_win_home: Regex,
}

impl ExportSanitizer {
    /// Create a new sanitizer with explicit rules and salt.
    pub fn new(rules: SanitizerRules, salt: Salt) -> Self {
        // Regex patterns are intentionally simple / conservative.
        let re_ip = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("valid IPv4 regex");
        let re_mac =
            Regex::new(r"\b[0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2}){5}\b").expect("valid MAC regex");
        let re_unix_home =
            Regex::new(r"(?P<prefix>/home/)(?P<user>[^/]+)").expect("valid unix home regex");
        let re_win_home =
            Regex::new(r"(?P<prefix>C:\\Users\\)(?P<user>[^\\]+)").expect("valid win home regex");

        ExportSanitizer {
            rules,
            salt,
            re_ip,
            re_mac,
            re_unix_home,
            re_win_home,
        }
    }

    /// The rules used by this sanitizer.
    pub fn rules(&self) -> &SanitizerRules {
        &self.rules
    }

    /// Core entry point: sanitize the given JSON packet in-place.
    /// Returns a report describing all redactions.
    pub fn sanitize_packet(&self, packet: &mut Value) -> SanitizationReport {
        if self.is_trusted(packet) {
            return SanitizationReport::trusted_skip(self.rules.paranoid);
        }

        let mut report = SanitizationReport::new(self.rules.paranoid);
        self.sanitize_value(packet, "$", &mut report);
        report
    }

    fn is_trusted(&self, packet: &Value) -> bool {
        if let Value::Object(root) = packet {
            if let Some(meta) = root.get("meta") {
                if let Value::Object(meta_map) = meta {
                    if let Some(Value::Bool(true)) = meta_map.get("trusted") {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn sanitize_value(&self, value: &mut Value, path: &str, report: &mut SanitizationReport) {
        match value {
            Value::Object(map) => self.sanitize_object(map, path, report),
            Value::Array(arr) => {
                for (idx, v) in arr.iter_mut().enumerate() {
                    let child_path = format!("{path}[{idx}]");
                    self.sanitize_value(v, &child_path, report);
                }
            }
            Value::String(s) => {
                let original = s.clone();
                let sanitized = self.mask_sensitive_fields(&original, path, report);
                if sanitized != original {
                    *s = sanitized;
                } else if self.rules.paranoid {
                    // Deep redact: hash the entire string if not obviously safe.
                    if !self.looks_safe_literal(&original) {
                        let replacement = self.hash_or_redact(&original);
                        report.redactions.push(RedactionEvent {
                            path: path.to_string(),
                            original_preview: preview(&original),
                            replacement_preview: preview(&replacement),
                            kind: RedactionKind::Other,
                        });
                        *s = replacement;
                    }
                }
            }
            _ => { /* numbers, bools, null left untouched */ }
        }
    }

    fn sanitize_object(
        &self,
        map: &mut Map<String, Value>,
        path: &str,
        report: &mut SanitizationReport,
    ) {
        // Because we need mutable access to both keys and values, we collect keys first.
        let keys: Vec<String> = map.keys().cloned().collect();
        for key in keys {
            let child_path = format!("{path}.{}", key);
            let is_whitelisted_key = self.rules.is_key_whitelisted(&key);

            if let Some(v) = map.get_mut(&key) {
                // Special-case host / hostname: we never want the raw host leaving the box.
                if matches!(key.as_str(), "host" | "hostname") {
                    if let Value::String(orig) = v.take() {
                        let replacement = "overflow_node".to_string();
                        report.redactions.push(RedactionEvent {
                            path: child_path.clone(),
                            original_preview: preview(&orig),
                            replacement_preview: preview(&replacement),
                            kind: RedactionKind::Hostname,
                        });
                        *v = Value::String(replacement);
                    }
                    continue;
                }

                // Path-ish keys get special treatment.
                if matches!(
                    key.as_str(),
                    "path" | "file" | "filepath" | "file_path" | "command_line"
                ) {
                    if let Value::String(orig) = v {
                        if !self.rules.is_path_whitelisted(orig) {
                            let replaced = self.redact_path_like(orig, &child_path, report);
                            *orig = replaced;
                        }
                        continue;
                    }
                }

                if is_whitelisted_key {
                    // Still traverse children in case nested structures have sensitive values.
                    self.sanitize_value(v, &child_path, report);
                } else {
                    self.sanitize_value(v, &child_path, report);
                }
            }
        }
    }

    /// Mask file paths and embedded usernames for path-like strings.
    fn redact_path_like(
        &self,
        s: &str,
        path: &str,
        report: &mut SanitizationReport,
    ) -> String {
        let original = s.to_string();

        // First handle Unix-style /home/<user>
        let after_unix = self
            .re_unix_home
            .replace_all(&original, |caps: &regex::Captures| {
                let user = caps.name("user").map(|m| m.as_str()).unwrap_or("");
                let user_tag = format!("user_{}", self.hash_or_redact(user));
                let replacement = format!("/redacted/path/{user_tag}");
                report.redactions.push(RedactionEvent {
                    path: path.to_string(),
                    original_preview: preview(user),
                    replacement_preview: preview(&user_tag),
                    kind: RedactionKind::Username,
                });
                replacement
            })
            .to_string();

        // Then Windows-style C:\Users\<user>
        let after_win = self
            .re_win_home
            .replace_all(&after_unix, |caps: &regex::Captures| {
                let user = caps.name("user").map(|m| m.as_str()).unwrap_or("");
                let user_tag = format!("user_{}", self.hash_or_redact(user));
                let replacement = format!(r"C:\redacted\{user_tag}");
                report.redactions.push(RedactionEvent {
                    path: path.to_string(),
                    original_preview: preview(user),
                    replacement_preview: preview(&user_tag),
                    kind: RedactionKind::Username,
                });
                replacement
            })
            .to_string();

        if after_win != original {
            report.redactions.push(RedactionEvent {
                path: path.to_string(),
                original_preview: preview(&original),
                replacement_preview: preview(&after_win),
                kind: RedactionKind::FilePath,
            });
        }

        after_win
    }

    /// Detect and mask IPs / MACs inside arbitrary string.
    fn mask_sensitive_fields(
        &self,
        s: &str,
        path: &str,
        report: &mut SanitizationReport,
    ) -> String {
        let mut result = s.to_string();

        // IP addresses
        let mut ip_events = Vec::new();
        let tmp = self
            .re_ip
            .replace_all(&result, |caps: &regex::Captures| {
                let ip = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                if self.rules.is_ip_whitelisted(ip) {
                    ip.to_string()
                } else {
                    let hashed = format!("ip_{}", self.hash_or_redact(ip));
                    ip_events.push((ip.to_string(), hashed.clone()));
                    hashed
                }
            })
            .to_string();

        for (orig, repl) in ip_events {
            report.redactions.push(RedactionEvent {
                path: path.to_string(),
                original_preview: preview(&orig),
                replacement_preview: preview(&repl),
                kind: RedactionKind::IpAddress,
            });
        }

        result = tmp;

        // MAC addresses
        let mut mac_events = Vec::new();
        let tmp = self
            .re_mac
            .replace_all(&result, |caps: &regex::Captures| {
                let mac = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                let hashed = format!("mac_{}", self.hash_or_redact(mac));
                mac_events.push((mac.to_string(), hashed.clone()));
                hashed
            })
            .to_string();

        for (orig, repl) in mac_events {
            report.redactions.push(RedactionEvent {
                path: path.to_string(),
                original_preview: preview(&orig),
                replacement_preview: preview(&repl),
                kind: RedactionKind::MacAddress,
            });
        }

        result = tmp;

        result
    }

    /// Salted SHA-256 hash → hex string.
    pub fn hash_or_redact(&self, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.salt.as_bytes());
        hasher.update(value.as_bytes());
        let digest = hasher.finalize();
        hex_from_bytes(&digest)
    }

    /// Heuristic: some literals are obviously harmless and can pass in non-paranoid mode
    /// (e.g., "OK", "cpu", "INFO").
    fn looks_safe_literal(&self, s: &str) -> bool {
        if s.len() <= 3 {
            return true;
        }
        // Simple heuristic: all uppercase ASCII and no digits.
        if s.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c == '-') {
            return true;
        }
        false
    }
}

fn preview(s: &str) -> String {
    const MAX: usize = 32;
    if s.len() <= MAX {
        s.to_string()
    } else {
        format!("{}...", &s[..MAX])
    }
}

fn hex_from_bytes(bytes: &[u8]) -> String {
    // Manual hex to avoid extra dependencies.
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}
