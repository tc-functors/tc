use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    sync::{
        Arc,
        Mutex,
        OnceLock,
    },
};
use tokio::sync::OnceCell;

pub struct AsyncMemo<K, V> {
    inner: OnceLock<Mutex<HashMap<K, Arc<OnceCell<V>>>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> AsyncMemo<K, V> {
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    /// Returns the cached value for `key`, computing it via `f` exactly
    /// once across all concurrent callers.
    pub async fn get_or_init<F, Fut>(&self, key: K, f: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V>,
    {
        let map = self.inner.get_or_init(|| Mutex::new(HashMap::new()));
        let cell = {
            let mut guard = map.lock().unwrap();
            guard
                .entry(key)
                .or_insert_with(|| Arc::new(OnceCell::new()))
                .clone()
        };
        cell.get_or_init(f).await.clone()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{
            AtomicUsize,
            Ordering,
        },
        time::Duration,
    };
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn caches_same_key() {
        let memo: AsyncMemo<&'static str, u32> = AsyncMemo::new();
        let calls = AtomicUsize::new(0);
        let v1 = memo
            .get_or_init("k", || async {
                calls.fetch_add(1, Ordering::SeqCst);
                42
            })
            .await;
        let v2 = memo
            .get_or_init("k", || async {
                calls.fetch_add(1, Ordering::SeqCst);
                99
            })
            .await;
        assert_eq!(v1, 42);
        assert_eq!(v2, 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distinct_keys_each_run_once() {
        let memo: AsyncMemo<&'static str, u32> = AsyncMemo::new();
        assert_eq!(memo.get_or_init("a", || async { 1 }).await, 1);
        assert_eq!(memo.get_or_init("b", || async { 2 }).await, 2);
        assert_eq!(memo.get_or_init("a", || async { 99 }).await, 1);
    }

    /// Regression: the LAYER_AUTH cache key uses `Option<String>` so
    /// `None` and `Some("")` stay distinct — matching the asymmetric
    /// behaviour of `Auth::assume(None, _)` (short-circuit, no AWS
    /// call) vs `Auth::assume(Some(""), _)` (calls Auth::new). An
    /// earlier draft used `unwrap_or_default()` on the way into the
    /// key, collapsing both onto `""` and risking a wrong-cached-Auth
    /// return. This test pins the option-keyed case so a future
    /// well-meaning simplification breaks loudly here.
    #[tokio::test]
    async fn distinguishes_none_from_empty_string_in_option_key() {
        let memo: AsyncMemo<(Option<String>, Option<String>), &'static str> = AsyncMemo::new();
        assert_eq!(
            memo.get_or_init((None, None), || async { "none-none" })
                .await,
            "none-none"
        );
        assert_eq!(
            memo.get_or_init((Some(String::new()), None), || async { "empty-none" })
                .await,
            "empty-none"
        );
        assert_eq!(
            memo.get_or_init((None, Some(String::new())), || async { "none-empty" })
                .await,
            "none-empty"
        );
        assert_eq!(
            memo.get_or_init((Some(String::new()), Some(String::new())), || async {
                "empty-empty"
            })
            .await,
            "empty-empty"
        );
        assert_eq!(
            memo.get_or_init((None, None), || async { "should-be-cached" })
                .await,
            "none-none"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_same_key_runs_once() {
        let memo: Arc<AsyncMemo<&'static str, u32>> = Arc::new(AsyncMemo::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(8));

        let mut handles = vec![];
        for _ in 0..8 {
            let memo = memo.clone();
            let calls = calls.clone();
            let barrier = barrier.clone();
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                memo.get_or_init("k", || {
                    let calls = calls.clone();
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        7u32
                    }
                })
                .await
            }));
        }

        for h in handles {
            assert_eq!(h.await.unwrap(), 7);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
