mod http_server;

enum ExitCode {
    Success = 0,
    Failure = 1,
}

impl Into<i32> for ExitCode {
    fn into(self) -> i32 {
        self as i32
    }
}

pub async fn run() -> i32 {
    let addr = ([127, 0, 0, 1], 3030);
    let server = http_server::HTTPServer::new(addr.into());
    server.run().await.unwrap();
    return ExitCode::Success.into();
}

#[cfg(test)]
mod test {
    use super::run;

    #[ignore]
    #[tokio::test]
    async fn test_run() {
        assert_eq!(run().await, 0);
    }
}
