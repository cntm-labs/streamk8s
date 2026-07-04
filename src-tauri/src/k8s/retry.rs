use std::future::Future;
use tokio::time::{sleep, Duration};

pub async fn with_retry<F, Fut, T>(mut action: F) -> Result<T, kube::Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, kube::Error>>,
{
    let mut retries = 3;
    let mut delay = Duration::from_millis(500);

    loop {
        match action().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if retries == 0 || !is_transient(&e) {
                    return Err(e);
                }
                retries -= 1;
                sleep(delay).await;
                delay *= 2;
            }
        }
    }
}

fn is_transient(e: &kube::Error) -> bool {
    match e {
        kube::Error::Api(err_resp) => err_resp.code == 429 || err_resp.code >= 500,
        kube::Error::Auth(_) => false,
        kube::Error::HyperError(_) => true,
        kube::Error::Service(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use kube::core::ErrorResponse;

    #[tokio::test]
    async fn test_with_retry_success_first_try() {
        let attempts = AtomicUsize::new(0);

        let result = with_retry(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Ok::<_, kube::Error>("success")
        })
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_with_retry_transient_error_then_success() {
        let attempts = AtomicUsize::new(0);

        let result = with_retry(|| async {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(kube::Error::Api(ErrorResponse {
                    code: 500,
                    message: "Internal Server Error".to_string(),
                    reason: "ServerTimeout".to_string(),
                    status: "Failure".to_string(),
                }))
            } else {
                Ok::<_, kube::Error>("success")
            }
        })
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_with_retry_transient_error_exhausted() {
        let attempts = AtomicUsize::new(0);

        let result = with_retry(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err::<(), kube::Error>(kube::Error::Api(ErrorResponse {
                code: 503,
                message: "Service Unavailable".to_string(),
                reason: "ServiceUnavailable".to_string(),
                status: "Failure".to_string(),
            }))
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // 1 initial + 3 retries
    }

    #[tokio::test]
    async fn test_with_retry_permanent_error() {
        let attempts = AtomicUsize::new(0);

        let result = with_retry(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err::<(), kube::Error>(kube::Error::Auth("Unauthorized".to_string()))
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // Should fail immediately
    }
}
