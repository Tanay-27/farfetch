use std::fs;
use zed_extension_api as zed;
use zed_extension_api::{
    current_platform, Architecture, Os,
    latest_github_release, GithubReleaseOptions,
    http_client::{fetch, HttpMethod, HttpRequest, HttpResponse, RedirectPolicy},
    serde_json,
    DownloadedFileType,
    SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput, SlashCommandOutputSection,
    Worktree,
};

const GITHUB_REPO: &str = "Tanay-27/farfetch";
const BIN_PATH: &str = "bin/farfetch";
const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

struct FarfetchExtension;

struct RequestArgs {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

// ── binary management ────────────────────────────────────────────────────────

impl FarfetchExtension {
    fn target_triple(os: Os, arch: Architecture) -> Result<&'static str, String> {
        match (os, arch) {
            (Os::Mac, Architecture::Aarch64)   => Ok("aarch64-apple-darwin"),
            (Os::Mac, Architecture::X8664)     => Ok("x86_64-apple-darwin"),
            (Os::Linux, Architecture::Aarch64) => Ok("aarch64-unknown-linux-gnu"),
            (Os::Linux, Architecture::X8664)   => Ok("x86_64-unknown-linux-gnu"),
            _ => Err("farfetch: unsupported platform".into()),
        }
    }

    fn ensure_binary(&self) -> Result<String, String> {
        if fs::metadata(BIN_PATH).map(|m| m.is_file()).unwrap_or(false) {
            return Ok(BIN_PATH.to_string());
        }

        let (os, arch) = current_platform();
        let target = Self::target_triple(os, arch)?;
        let asset_name = format!("farfetch-{}.tar.gz", target);

        let release = latest_github_release(
            GITHUB_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "farfetch {}: no asset '{}' in release",
                    release.version, asset_name
                )
            })?;

        zed::download_file(&asset.download_url, BIN_PATH, DownloadedFileType::GzipTar)?;
        zed::make_file_executable(BIN_PATH)?;

        Ok(BIN_PATH.to_string())
    }

    // Phase B follow-up: output the one-time shell command that symlinks the
    // extension-cached binary into ~/.local/bin so `farfetch: Open` finds it.
    fn install_instructions(&self) -> Result<String, String> {
        self.ensure_binary()?;

        let (os, _) = current_platform();
        let src = match os {
            Os::Mac => {
                "\"$HOME/Library/Application Support/Zed/extensions/installed/farfetch/bin/farfetch\""
            }
            Os::Linux => {
                "\"$HOME/.local/share/zed/extensions/installed/farfetch/bin/farfetch\""
            }
            _ => return Err("unsupported platform".into()),
        };

        Ok(format!(
            "**farfetch binary downloaded.** \
            Run this once in your terminal to add it to PATH:\n\
            ```bash\n\
            mkdir -p ~/.local/bin\n\
            ln -sf {src} ~/.local/bin/farfetch\n\
            ```\n\
            After that, `~/.local/bin` must be in your `$PATH`. \
            Then `farfetch: Open` in the task palette launches the TUI directly."
        ))
    }
}

// ── HTTP request (Phase D) ───────────────────────────────────────────────────

impl FarfetchExtension {
    fn fire_request(args: &RequestArgs) -> Result<String, String> {
        let method = match args.method.as_str() {
            "GET"     => HttpMethod::Get,
            "POST"    => HttpMethod::Post,
            "PUT"     => HttpMethod::Put,
            "PATCH"   => HttpMethod::Patch,
            "DELETE"  => HttpMethod::Delete,
            "HEAD"    => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            m => return Err(format!(
                "unknown method '{}'. Use GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS", m
            )),
        };

        let mut builder = HttpRequest::builder()
            .method(method)
            .url(&args.url)
            .redirect_policy(RedirectPolicy::FollowLimit(5))
            .headers(args.headers.iter().cloned());

        if let Some(body) = &args.body {
            builder = builder.body(body.as_bytes().to_vec());
        }

        let request = builder.build()?;
        let response = fetch(&request)?;
        Ok(format_response(&args.method, &args.url, response))
    }
}

// ── argument parser ──────────────────────────────────────────────────────────

fn parse_request_args(args: &[String]) -> Result<RequestArgs, String> {
    let mut iter = args.iter();

    let method = iter
        .next()
        .ok_or("usage: /farfetch <METHOD> <URL> [-H \"Name: Value\"]* [-d <body>]")?
        .to_uppercase();

    let url = iter
        .next()
        .ok_or("usage: /farfetch <METHOD> <URL>")?
        .clone();

    let rest: Vec<_> = iter.collect();
    let mut headers = Vec::new();
    let mut body = None;
    let mut i = 0;

    while i < rest.len() {
        match rest[i].as_str() {
            "-H" | "--header" => {
                i += 1;
                let hdr = rest.get(i).ok_or("-H requires a value, e.g. -H \"Content-Type: application/json\"")?;
                let (name, value) = hdr
                    .split_once(": ")
                    .or_else(|| hdr.split_once(':'))
                    .ok_or_else(|| format!("invalid header '{}' — expected 'Name: Value'", hdr))?;
                headers.push((name.trim().to_string(), value.trim().to_string()));
                i += 1;
            }
            "-d" | "--data" => {
                i += 1;
                body = Some((*rest.get(i).ok_or("-d requires a value")?).clone());
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    Ok(RequestArgs { method, url, headers, body })
}

// ── response formatter ───────────────────────────────────────────────────────

fn format_response(method: &str, url: &str, response: HttpResponse) -> String {
    let content_type = response
        .headers
        .iter()
        .find(|(k, _)| k.to_lowercase() == "content-type")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    // HTTP/2 pseudo-headers carry the status code as ":status"
    let status = response
        .headers
        .iter()
        .find(|(k, _)| k == ":status")
        .map(|(_, v)| format!(" {}", v))
        .unwrap_or_default();

    let body_str = match String::from_utf8(response.body) {
        Ok(s) => s,
        Err(_) => return format!("**{} {}{}** — binary (non-UTF-8) response body", method, url, status),
    };

    let is_json = content_type.contains("json");
    let formatted_body = if is_json {
        serde_json::from_str::<serde_json::Value>(&body_str)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or_else(|| body_str.clone())
    } else {
        body_str
    };

    let lang = if is_json { "json" } else { "" };

    format!(
        "**{} {}{}**\n\n```{}\n{}\n```",
        method, url, status, lang, formatted_body
    )
}

// ── Extension trait ──────────────────────────────────────────────────────────

impl zed::Extension for FarfetchExtension {
    fn new() -> Self {
        FarfetchExtension
    }

    fn complete_slash_command_argument(
        &self,
        _command: SlashCommand,
        args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        let prefix = args.first().map(|s| s.as_str()).unwrap_or("");

        if args.len() <= 1 {
            let mut completions: Vec<_> = HTTP_METHODS
                .iter()
                .filter(|m| m.starts_with(&prefix.to_uppercase().as_str()))
                .map(|m| SlashCommandArgumentCompletion {
                    label: format!("{} <url>", m),
                    new_text: format!("{} ", m),
                    run_command: false,
                })
                .collect();

            for sub in ["install", "version", "status"] {
                if sub.starts_with(prefix) {
                    completions.push(SlashCommandArgumentCompletion {
                        label: sub.into(),
                        new_text: sub.into(),
                        run_command: true,
                    });
                }
            }
            return Ok(completions);
        }

        Ok(vec![])
    }

    fn run_slash_command(
        &self,
        _command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let first = args.first().map(|s| s.as_str()).unwrap_or("status");

        let text = if HTTP_METHODS.iter().any(|m| m.eq_ignore_ascii_case(first)) {
            let parsed = parse_request_args(&args)?;
            Self::fire_request(&parsed)?
        } else {
            match first {
                "install" => self.install_instructions()?,
                "version" => {
                    self.ensure_binary()?;
                    "farfetch extension binary ready.".to_string()
                }
                "status" | _ => match self.ensure_binary() {
                    Ok(_) => "**farfetch** — binary ready.\n\n\
                        - `/farfetch install` — symlink to `~/.local/bin` for the TUI task\n\
                        - `/farfetch GET <url>` — fire a request inline\n\
                        - `/farfetch POST <url> -H \"Content-Type: application/json\" -d '{}'`\n\
                        - Task palette → **farfetch: Open** — full TUI"
                        .to_string(),
                    Err(e) => format!(
                        "farfetch binary not available: {}\n\nRun `/farfetch install` to download it.",
                        e
                    ),
                },
            }
        };

        let len = text.len() as u32;
        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..len).into(),
                label: "farfetch".into(),
            }],
            text,
        })
    }
}

zed::register_extension!(FarfetchExtension);
