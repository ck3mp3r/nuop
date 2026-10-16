use super::support::*;

#[tokio::test]
async fn test_error_policy() {
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
    let error = Error::Api(
        kube::core::Status::failure("Test error", "TestReason")
            .with_code(500)
            .boxed(),
    );

    let result = error_policy(obj, &error, state);
    assert_eq!(result, Action::requeue(Duration::from_secs(300)));
}

#[tokio::test]
async fn test_detect_phase_combinations() {
    // Test all phase detection combinations
    let obj_no_finalizer = create_test_object("test", "default", false, false);
    let obj_with_finalizer = create_test_object("test", "default", true, false);
    let obj_deleting_with_finalizer = create_test_object("test", "default", true, true);
    let obj_deleting_no_finalizer = create_test_object("test", "default", false, true);

    // No finalizer configured
    assert_eq!(
        detect_phase(&obj_no_finalizer, None),
        ReconcilePhase::Noop("reconcile")
    );
    assert_eq!(
        detect_phase(&obj_with_finalizer, None),
        ReconcilePhase::Noop("reconcile")
    );

    // With finalizer configured
    let finalizer = Some("test.example.com/finalizer");
    assert_eq!(
        detect_phase(&obj_no_finalizer, finalizer),
        ReconcilePhase::NeedsFinalizer
    );
    assert_eq!(
        detect_phase(&obj_with_finalizer, finalizer),
        ReconcilePhase::Active
    );
    assert_eq!(
        detect_phase(&obj_deleting_with_finalizer, finalizer),
        ReconcilePhase::Finalizing
    );
    assert_eq!(
        detect_phase(&obj_deleting_no_finalizer, finalizer),
        ReconcilePhase::NeedsFinalizer
    );
}

#[tokio::test]
async fn test_api_error_handling() {
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

    // Mock API error response
    tokio::spawn(async move {
        let (_request, send_response) = handle.next_request().await.expect("service not called");

        let error_response = json!({
            "kind": "Status",
            "apiVersion": "v1",
            "metadata": {},
            "status": "Failure",
            "message": "deployments.apps \"test-deployment\" not found",
            "reason": "NotFound",
            "code": 404
        });

        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
            .unwrap();
        send_response.send_response(response);
    });

    let result = reconcile(obj.clone(), state).await;
    assert!(result.is_err());
}
