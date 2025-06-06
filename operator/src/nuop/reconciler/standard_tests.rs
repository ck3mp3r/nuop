use super::standard::get_standard_controllers;
use kube::Client;
use std::path::PathBuf;

// Helper function to create a test kube client
// Since we're only testing the controller creation logic, not actual API calls,
// we can use a simple mock that won't be called during our tests
fn create_test_client() -> Client {
    // Create a mock service that we won't actually use in these tests
    use http::{Request, Response};
    use kube::client::Body;
    use tower_test::mock;

    let (mock_service, _handle) = mock::pair::<Request<Body>, Response<Body>>();
    Client::new(mock_service, "default")
}

fn get_test_scripts() -> Vec<PathBuf> {
    vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/configmap-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/secret-controller.nu"),
    ]
}

fn get_duplicate_scripts() -> Vec<PathBuf> {
    vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/duplicate-pod-controller.nu"), // Same kind as pod-controller
    ]
}

#[tokio::test]
async fn test_successful_standard_controller_creation() {
    let client = create_test_client();
    let scripts = get_test_scripts();

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 5 controllers (one for each unique kind)
    assert_eq!(controllers.len(), 5);

    // Clean up by aborting the spawned tasks
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_with_duplicate_kinds() {
    let client = create_test_client();
    let scripts = get_duplicate_scripts();

    let controllers = get_standard_controllers(&client, &scripts);

    // Should only create 1 controller (duplicate Pod kind should be rejected)
    assert_eq!(controllers.len(), 1);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_with_invalid_script_config() {
    let client = create_test_client();

    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/invalid-config-controller.nu"), // Returns invalid config
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"),
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should only create 2 controllers (invalid script config should be skipped)
    assert_eq!(controllers.len(), 2);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_with_nonexistent_script() {
    let client = create_test_client();

    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"),
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/nonexistent-controller.nu"), // Doesn't exist
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"),
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should only create 2 controllers (nonexistent script should be skipped)
    assert_eq!(controllers.len(), 2);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_with_empty_scripts() {
    let client = create_test_client();
    let scripts: Vec<PathBuf> = vec![];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create no controllers
    assert_eq!(controllers.len(), 0);
}

#[tokio::test]
async fn test_mixed_valid_and_invalid_scripts() {
    let client = create_test_client();

    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Valid
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/invalid-config-controller.nu"), // Invalid config
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"), // Valid
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/nonexistent-controller.nu"), // Doesn't exist
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"), // Valid
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 3 controllers (only valid scripts)
    assert_eq!(controllers.len(), 3);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_multiple_duplicates_same_kind() {
    let client = create_test_client();

    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Pod kind
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/duplicate-pod-controller.nu"), // Also Pod kind
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"), // Deployment kind
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 2 controllers (first Pod and Deployment, second Pod should be rejected)
    assert_eq!(controllers.len(), 2);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_controller_task_spawning() {
    let client = create_test_client();
    let scripts = vec![PathBuf::from(
        "src/nuop/reconciler/standard_tests/scripts/pod-controller.nu",
    )];

    let controllers = get_standard_controllers(&client, &scripts);
    assert_eq!(controllers.len(), 1);

    // Test that the task is actually running by checking if it's not finished immediately
    let controller = &controllers[0];
    assert!(!controller.is_finished());

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_different_resource_kinds() {
    let client = create_test_client();

    // Test with various Kubernetes resource kinds
    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Core/v1 Pod
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"), // apps/v1 Deployment
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"), // Core/v1 Service
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/configmap-controller.nu"), // Core/v1 ConfigMap
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/secret-controller.nu"), // Core/v1 Secret
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 5 controllers (all different kinds)
    assert_eq!(controllers.len(), 5);

    // All controllers should be running
    for controller in &controllers {
        assert!(!controller.is_finished());
    }

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_kind_deduplication_logic() {
    let client = create_test_client();

    // Create multiple scripts with same kind but different configs
    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Pod - should be accepted (first)
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"), // Deployment - should be accepted
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/duplicate-pod-controller.nu"), // Pod - should be rejected (duplicate)
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"), // Service - should be accepted
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 3 controllers (Pod, Deployment, Service - duplicate Pod rejected)
    assert_eq!(controllers.len(), 3);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_script_config_variations() {
    let client = create_test_client();

    // Test scripts with different configurations (finalizers, namespaces, selectors)
    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Has finalizer, namespace, selectors
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"), // No finalizer, no namespace, no selectors
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/secret-controller.nu"), // Has finalizer and namespace
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 3 controllers regardless of config variations
    assert_eq!(controllers.len(), 3);

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_error_resilience() {
    let client = create_test_client();

    // Mix of valid scripts, invalid configs, and missing files
    let scripts = vec![
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/pod-controller.nu"), // Valid
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/invalid-config-controller.nu"), // Invalid config
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/missing-file.nu"), // Missing file
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/deployment-controller.nu"), // Valid
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/another-missing.nu"), // Another missing file
        PathBuf::from("src/nuop/reconciler/standard_tests/scripts/service-controller.nu"), // Valid
    ];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 3 controllers (only the valid ones)
    assert_eq!(controllers.len(), 3);

    // All controllers should be running
    for controller in &controllers {
        assert!(!controller.is_finished());
    }

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}

#[tokio::test]
async fn test_single_script() {
    let client = create_test_client();
    let scripts = vec![PathBuf::from(
        "src/nuop/reconciler/standard_tests/scripts/pod-controller.nu",
    )];

    let controllers = get_standard_controllers(&client, &scripts);

    // Should create 1 controller
    assert_eq!(controllers.len(), 1);

    // Controller should be running
    assert!(!controllers[0].is_finished());

    // Clean up
    for controller in controllers {
        controller.abort();
    }
}
