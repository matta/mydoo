use anyhow::{Context, Result};
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub(crate) struct TokenSpan {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedRustFile {
    pub content: String,
    pub tokens: Vec<TokenSpan>,
    pub non_trivia_token_count: usize,
}
type CacheMap = HashMap<PathBuf, Arc<ParsedRustFile>>;
type SharedCache = Mutex<CacheMap>;

static PARSE_CACHE: OnceLock<SharedCache> = OnceLock::new();

pub(crate) fn parse_rust_source(content: &str) -> ParsedRustFile {
    let mut offset = 0usize;
    let mut non_trivia_token_count = 0usize;

    let tokens = ra_ap_rustc_lexer::tokenize(content, FrontmatterAllowed::Yes)
        .map(|token| {
            let start = offset;
            let len = token.len as usize;
            let end = start + len;
            offset = end;

            if !matches!(
                token.kind,
                TokenKind::LineComment { .. }
                    | TokenKind::BlockComment { .. }
                    | TokenKind::Whitespace
            ) {
                non_trivia_token_count += 1;
            }

            TokenSpan {
                kind: token.kind,
                start,
                end,
            }
        })
        .collect();

    ParsedRustFile {
        content: content.to_string(),
        tokens,
        non_trivia_token_count,
    }
}

pub(crate) fn parse_rust_file_cached(path: &Path) -> Result<Arc<ParsedRustFile>> {
    let cache = PARSE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    {
        let cache_guard = cache
            .lock()
            .expect("rust token cache mutex should not be poisoned");
        if let Some(parsed) = cache_guard.get(path) {
            return Ok(Arc::clone(parsed));
        }
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let parsed = Arc::new(parse_rust_source(&content));

    let mut cache_guard = cache
        .lock()
        .expect("rust token cache mutex should not be poisoned");
    cache_guard.insert(path.to_path_buf(), Arc::clone(&parsed));

    Ok(parsed)
}
