use secrecy::SecretString;
use see_see_lib::{error::ErrorCode, providers, settings::WebdavSettings, webdav};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{basic_auth, body_bytes, method, path},
};

fn connection(server: &MockServer, root: &str) -> WebdavSettings {
    WebdavSettings {
        url: format!("{}/dav/", server.uri()),
        username: "sync-user".into(),
        remote_root: root.into(),
        has_password: true,
    }
}

#[tokio::test]
async fn uploads_and_downloads_in_nested_root_with_basic_auth() {
    let server = MockServer::start().await;
    let payload = br#"{"version":1,"models":[],"prompts":[],"activeModelConfigId":null}"#;
    for (directory, status) in [("/dav/shared/", 405), ("/dav/shared/see-see/", 201)] {
        Mock::given(method("MKCOL"))
            .and(path(directory))
            .and(basic_auth("sync-user", " test-password "))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("PUT"))
        .and(path("/dav/shared/see-see/see-see-config.json"))
        .and(basic_auth("sync-user", " test-password "))
        .and(body_bytes(payload.as_slice()))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dav/shared/see-see/see-see-config.json"))
        .and(basic_auth("sync-user", " test-password "))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(payload))
        .expect(1)
        .mount(&server)
        .await;

    let settings = connection(&server, "shared/see-see");
    let password = SecretString::from(" test-password ");
    let client = providers::client().unwrap();
    webdav::upload(&client, &settings, Some(&password), payload)
        .await
        .unwrap();
    assert_eq!(
        webdav::download(&client, &settings, Some(&password))
            .await
            .unwrap(),
        payload
    );
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 4);
    assert!(
        requests
            .iter()
            .all(|request| request.url.path().starts_with("/dav/shared/"))
    );
}

#[tokio::test]
async fn endpoint_root_and_unicode_segments_are_encoded_as_path_components() {
    let server = MockServer::start().await;
    let settings = WebdavSettings {
        url: server.uri(),
        username: String::new(),
        remote_root: "我的配置/a b".into(),
        has_password: false,
    };
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .expect(1)
        .mount(&server)
        .await;
    webdav::download(&providers::client().unwrap(), &settings, None)
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests[0].url.path(),
        "/%E6%88%91%E7%9A%84%E9%85%8D%E7%BD%AE/a%20b/see-see-config.json"
    );
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn authentication_and_directory_failures_never_attempt_put() {
    for status in [401, 403, 409, 507] {
        let server = MockServer::start().await;
        Mock::given(method("MKCOL"))
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        let result = webdav::upload(
            &providers::client().unwrap(),
            &connection(&server, "see-see"),
            Some(&SecretString::from("test-password")),
            b"{}",
        )
        .await;
        assert!(result.is_err());
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
        if status == 401 || status == 403 {
            assert_eq!(result.unwrap_err().code, ErrorCode::AuthFailed);
        }
    }
}

#[tokio::test]
async fn missing_remote_file_has_actionable_error_without_server_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_string("private server response"))
        .mount(&server)
        .await;
    let error = webdav::download(
        &providers::client().unwrap(),
        &connection(&server, "see-see"),
        Some(&SecretString::from("test-password")),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(!error.message.contains("private server response"));
    assert!(error.details.is_none());
}

#[tokio::test]
async fn redirects_are_not_followed_with_credentials() {
    let destination = MockServer::start().await;
    let source = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(307).insert_header("Location", destination.uri()))
        .mount(&source)
        .await;
    let error = webdav::download(
        &providers::client().unwrap(),
        &connection(&source, "see-see"),
        Some(&SecretString::from("test-password")),
    )
    .await
    .unwrap_err();
    assert!(error.message.contains("重定向"));
    assert!(destination.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn missing_password_and_oversize_upload_fail_before_remote_write() {
    let server = MockServer::start().await;
    let client = providers::client().unwrap();
    let settings = connection(&server, "see-see");
    assert!(
        webdav::upload(&client, &settings, None, b"{}")
            .await
            .is_err()
    );
    assert!(
        webdav::upload(
            &client,
            &settings,
            Some(&SecretString::from("test-password")),
            &vec![b'x'; webdav::MAX_SNAPSHOT_BYTES + 1],
        )
        .await
        .is_err()
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn oversize_download_is_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(vec![b'x'; webdav::MAX_SNAPSHOT_BYTES + 1]),
        )
        .mount(&server)
        .await;
    let error = webdav::download(
        &providers::client().unwrap(),
        &connection(&server, "see-see"),
        Some(&SecretString::from("test-password")),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert!(error.message.contains("过大"));
}
