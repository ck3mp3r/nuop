use super::support::*;

// Comprehensive test that covers multiple edge cases and error scenarios
#[tokio::test]
async fn test_comprehensive_reconcile_scenarios() {
    if should_skip_script_tests() {
        return;
    }
    // Test 1: Finalizer already exists, no duplicate addition
    {
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

        // Object already has the correct finalizer
        let mut obj = create_test_object("test-deployment", "default", true, false);
        obj.metadata.finalizers = Some(vec!["test.example.com/finalizer".to_string()]);
        let obj = Arc::new(obj);

        let result = reconcile(obj.clone(), state).await.unwrap();
        assert_eq!(result, Action::requeue(Duration::from_secs(300))); // Should run as active, no changes
    }

    // Test 2: Script spawn failure
    {
        let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "default");

        let config = create_test_config();
        let api_resource = ApiResource::from_gvk(&(&config).into());
        let state = Arc::new(State::new_default(
            api_resource.clone(),
            client,
            config.clone(),
            PathBuf::from("/nonexistent/script"), // Non-existent script
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
            assert_eq!(error_response.code, 500);
            assert_eq!(error_response.message, "Failed to execute script");
        } else {
            panic!("Expected API error for script not found");
        }
    }

    // Test 3: Different resource types (not just Deployment)
    {
        let (mock_service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "default");

        let mut config = create_test_config();
        config.group = "".to_string(); // Core API group
        config.kind = "ConfigMap".to_string();

        let script = get_test_script_path("configmap");
        let api_resource = ApiResource::from_gvk(&(&config).into());
        let state = Arc::new(State::new_default(
            api_resource.clone(),
            client,
            config.clone(),
            script,
        ));

        let mut obj = DynamicObject::new(
            "test-configmap",
            &ApiResource::from_gvk(&GroupVersionKind {
                group: "".to_string(),
                version: "v1".to_string(),
                kind: "ConfigMap".to_string(),
            }),
        );

        obj.metadata = ObjectMeta {
            name: Some("test-configmap".to_string()),
            namespace: Some("default".to_string()),
            uid: Some("test-uid-456".to_string()),
            resource_version: Some("456".to_string()),
            finalizers: None,
            deletion_timestamp: None,
            ..Default::default()
        };

        obj.data = json!({
            "data": {
                "key": "value"
            }
        });

        let obj = Arc::new(obj);

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
                    .contains("/api/v1/namespaces/default/configmaps/test-configmap")
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
}

// Test multiple API failures and retries
#[tokio::test]
async fn test_api_failure_scenarios() {
    // Test 1: 409 Conflict during finalizer addition
    {
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

        tokio::spawn(async move {
            let (_request, send_response) =
                handle.next_request().await.expect("service not called");

            let error_response = json!({
                "kind": "Status",
                "apiVersion": "v1",
                "metadata": {},
                "status": "Failure",
                "message": "Operation cannot be fulfilled on deployments.apps \"test-deployment\": the object has been modified",
                "reason": "Conflict",
                "code": 409
            });

            let response = Response::builder()
                .status(StatusCode::CONFLICT)
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap();
            send_response.send_response(response);
        });

        let result = reconcile(obj.clone(), state).await;
        assert!(result.is_err());
        if let Err(Error::Api(error_response)) = result {
            assert_eq!(error_response.code, 500);
            assert_eq!(error_response.message, "Failed to add finalizer");
        } else {
            panic!("Expected API error for finalizer addition failure");
        }
    }

    // Test 2: Finalization with API error during finalizer removal
    {
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

        tokio::spawn(async move {
            let (_request, send_response) =
                handle.next_request().await.expect("service not called");

            let error_response = json!({
                "kind": "Status",
                "apiVersion": "v1",
                "metadata": {},
                "status": "Failure",
                "message": "Internal server error",
                "reason": "InternalError",
                "code": 500
            });

            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap();
            send_response.send_response(response);
        });

        let result = reconcile(obj.clone(), state).await;
        assert!(result.is_err());
        if let Err(Error::Api(error_response)) = result {
            assert_eq!(error_response.code, 500);
        } else {
            panic!("Expected API server error");
        }
    }
}

// Test edge cases with script execution
#[tokio::test]
async fn test_script_execution_edge_cases() {
    if should_skip_script_tests() {
        return;
    }
    // Test 1: Script with unexpected exit codes
    {
        let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "default");

        let config = create_test_config();
        let script = get_test_script_path("unexpected-exit-code");
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
            assert_eq!(error_response.code, 42);
            assert_eq!(error_response.message, "Script exited with error");
        } else {
            panic!("Expected API error for unexpected exit code");
        }
    }

    // Test 2: Script during finalization fails
    {
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

        let obj = Arc::new(create_test_object("test-deployment", "default", true, true));

        let result = reconcile(obj.clone(), state).await;
        assert!(result.is_err());
        if let Err(Error::Api(error_response)) = result {
            assert_eq!(error_response.code, 1);
            assert_eq!(error_response.message, "Script exited with error");
        } else {
            panic!("Expected API error for finalize script failure");
        }
    }
}

// Test configuration variations
#[tokio::test]
async fn test_configuration_variations() {
    if should_skip_script_tests() {
        return;
    }
    // Test 1: Empty namespace (should use object namespace)
    {
        let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "default");

        let mut config = create_test_config();
        config.namespace = None; // No namespace restriction
        config.finalizer = None; // No finalizer

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
            "kube-system",
            false,
            false,
        ));

        let result = reconcile(obj.clone(), state).await.unwrap();
        assert_eq!(result, Action::requeue(Duration::from_secs(10))); // Should use requeue_after_change
    }

    // Test 2: Custom requeue times
    {
        let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "default");

        let mut config = create_test_config();
        config.requeue_after_change = 60;
        config.requeue_after_noop = 600;
        config.finalizer = None;

        let script = get_test_script_path("custom-requeue");
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
        assert_eq!(result, Action::requeue(Duration::from_secs(600))); // Custom requeue_after_noop
    }
}
