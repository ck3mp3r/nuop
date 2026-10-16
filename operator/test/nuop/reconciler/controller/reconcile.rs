use super::support::*;

#[tokio::test]
async fn test_reconcile_needs_finalizer() {
    let (mock_service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let config = create_test_config();
    let script = get_test_script_path("success-no-changes");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object(
        "test-deployment",
        "default",
        false,
        false,
    ));

    // Mock the API response for adding finalizer
    let response_obj = {
        let mut obj = obj.as_ref().clone();
        obj.metadata.finalizers = Some(vec!["test.example.com/finalizer".to_string()]);
        obj
    };

    tokio::spawn(async move {
        let (request, send_response) = handle.next_request().await.expect("service not called");
        assert_eq!(request.method(), "PUT");
        assert!(
            request
                .uri()
                .path()
                .contains("/apis/apps/v1/namespaces/default/deployments/test-deployment")
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(serde_json::to_vec(&response_obj).unwrap()))
            .unwrap();
        send_response.send_response(response);
    });

    let result = reconcile(obj.clone(), state).await.unwrap();
    assert_eq!(result, Action::requeue(Duration::from_secs(5)));
}

#[tokio::test]
async fn test_reconcile_active_no_changes() {
    if should_skip_script_tests() {
        return;
    }
    let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let config = create_test_config();
    let script = get_test_script_path("success-no-changes");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object(
        "test-deployment",
        "default",
        true,
        false,
    ));

    let result = reconcile(obj.clone(), state).await.unwrap();
    assert_eq!(result, Action::requeue(Duration::from_secs(300))); // requeue_after_noop
}

#[tokio::test]
async fn test_reconcile_active_with_changes() {
    if should_skip_script_tests() {
        return;
    }
    let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let config = create_test_config();
    let script = get_test_script_path("success-with-changes");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object(
        "test-deployment",
        "default",
        true,
        false,
    ));

    let result = reconcile(obj.clone(), state).await.unwrap();
    assert_eq!(result, Action::requeue(Duration::from_secs(10))); // requeue_after_change
}

#[tokio::test]
async fn test_reconcile_finalizing() {
    if should_skip_script_tests() {
        return;
    }
    let (mock_service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let config = create_test_config();
    let script = get_test_script_path("success-no-changes");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object("test-deployment", "default", true, true));

    // Mock the API response for removing finalizer
    let response_obj = {
        let mut obj = obj.as_ref().clone();
        obj.metadata.finalizers = Some(vec![]); // Finalizer removed
        obj
    };

    tokio::spawn(async move {
        let (request, send_response) = handle.next_request().await.expect("service not called");
        assert_eq!(request.method(), "PUT");
        assert!(
            request
                .uri()
                .path()
                .contains("/apis/apps/v1/namespaces/default/deployments/test-deployment")
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(serde_json::to_vec(&response_obj).unwrap()))
            .unwrap();
        send_response.send_response(response);
    });

    let result = reconcile(obj.clone(), state).await.unwrap();
    assert_eq!(result, Action::await_change());
}

#[tokio::test]
async fn test_reconcile_script_error() {
    if should_skip_script_tests() {
        return;
    }
    let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let config = create_test_config();
    let script = get_test_script_path("error");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object(
        "test-deployment",
        "default",
        true,
        false,
    ));

    let result = reconcile(obj.clone(), state).await;
    assert!(result.is_err());

    if let Err(Error::Api(error_response)) = result {
        assert_eq!(error_response.code, 1);
        assert_eq!(error_response.message, "Script exited with error");
    } else {
        panic!("Expected API error");
    }
}

#[tokio::test]
async fn test_reconcile_no_finalizer_config() {
    if should_skip_script_tests() {
        return;
    }
    let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
    let client = Client::new(mock_service, "default");

    let mut config = create_test_config();
    config.finalizer = None; // No finalizer configured
    let script = get_test_script_path("no-finalizer");
    let api_resource = ApiResource::from_gvk(&(&config).into());
    let state = Arc::new(State::new_default(
        api_resource.clone(),
        client,
        config.clone(),
        script,
    ));

    let obj = Arc::new(create_test_object(
        "test-deployment",
        "default",
        false,
        false,
    ));

    let result = reconcile(obj.clone(), state).await.unwrap();
    assert_eq!(result, Action::requeue(Duration::from_secs(300))); // Should run as noop with reconcile command
}
