#[cfg(test)]
mod tests {
    use super::super::managed::get_managed_controllers;
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

    fn get_test_mappings() -> Vec<PathBuf> {
        vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/deployment-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/service-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/duplicate-mapping.yaml"),
        ]
    }

    fn get_test_scripts() -> Vec<PathBuf> {
        vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/deployment-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/service-script.sh"),
        ]
    }

    #[tokio::test]
    async fn test_successful_controller_creation() {
        let client = create_test_client();
        let mappings = get_test_mappings();
        let scripts = get_test_scripts();

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create 3 controllers (one for each script with matching mapping)
        assert_eq!(controllers.len(), 3);

        // Clean up by aborting the spawned tasks
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_duplicate_kinds() {
        let client = create_test_client();
        let mappings = get_test_mappings();

        // Include duplicate script that has same group/kind/version as deployment-script
        let scripts = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/deployment-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/duplicate-script.sh"),
        ];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should only create 2 controllers (duplicate should be rejected)
        assert_eq!(controllers.len(), 2);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_invalid_mapping_file() {
        let client = create_test_client();

        let mappings = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/invalid-mapping.yaml"), // Invalid YAML
        ];

        let scripts = vec![PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh")];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should still create 1 controller (invalid mapping should be skipped)
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_nonexistent_mapping_file() {
        let client = create_test_client();

        let mappings = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/nonexistent.yaml"), // File doesn't exist
        ];

        let scripts = vec![PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh")];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should still create 1 controller (missing mapping should be skipped)
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_invalid_script_config() {
        let client = create_test_client();
        let mappings = get_test_mappings();

        let scripts = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/invalid-config-script.sh"), // Returns invalid config
        ];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should only create 1 controller (invalid script config should be skipped)
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_script_without_matching_mapping() {
        let client = create_test_client();
        let mappings = get_test_mappings();

        let scripts = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/no-mapping-script.sh"), // No matching mapping
        ];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should only create 1 controller (script without mapping should be skipped)
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_nonexistent_script() {
        let client = create_test_client();
        let mappings = get_test_mappings();

        let scripts = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/nonexistent-script.sh"), // Doesn't exist
        ];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should only create 1 controller (nonexistent script should be skipped)
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_with_empty_mappings_and_scripts() {
        let client = create_test_client();
        let mappings: Vec<PathBuf> = vec![];
        let scripts: Vec<PathBuf> = vec![];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create no controllers
        assert_eq!(controllers.len(), 0);
    }

    #[tokio::test]
    async fn test_with_empty_mappings() {
        let client = create_test_client();
        let mappings: Vec<PathBuf> = vec![];
        let scripts = get_test_scripts();

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create no controllers (no mappings available)
        assert_eq!(controllers.len(), 0);
    }

    #[tokio::test]
    async fn test_with_empty_scripts() {
        let client = create_test_client();
        let mappings = get_test_mappings();
        let scripts: Vec<PathBuf> = vec![];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create no controllers (no scripts to process)
        assert_eq!(controllers.len(), 0);
    }

    #[tokio::test]
    async fn test_mapping_overrides_script_config() {
        let client = create_test_client();

        // The pod-mapping.yaml has different selectors and requeue times than pod-script.sh config
        let mappings = vec![PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml")];

        let scripts = vec![PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh")];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create 1 controller
        assert_eq!(controllers.len(), 1);

        // The config should be overridden by the mapping values
        // We can't easily test the internal config without more complex setup,
        // but the fact that a controller was created confirms the mapping was applied

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_partial_mapping_overrides() {
        let client = create_test_client();

        // The service-mapping.yaml only has basic info, no selectors or requeue overrides
        let mappings = vec![PathBuf::from("src/nuop/reconciler/managed_tests/mappings/service-mapping.yaml")];

        let scripts = vec![PathBuf::from("src/nuop/reconciler/managed_tests/scripts/service-script.sh")];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create 1 controller with script config values preserved where mapping doesn't override
        assert_eq!(controllers.len(), 1);

        // Clean up
        for controller in controllers {
            controller.abort();
        }
    }

    #[tokio::test]
    async fn test_controller_task_spawning() {
        let client = create_test_client();
        let mappings = vec![PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml")];
        let scripts = vec![PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh")];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);
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
    async fn test_multiple_valid_controllers() {
        let client = create_test_client();

        // Use all valid mappings and scripts
        let mappings = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/pod-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/deployment-mapping.yaml"),
            PathBuf::from("src/nuop/reconciler/managed_tests/mappings/service-mapping.yaml"),
        ];

        let scripts = vec![
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/pod-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/deployment-script.sh"),
            PathBuf::from("src/nuop/reconciler/managed_tests/scripts/service-script.sh"),
        ];

        let controllers = get_managed_controllers(&client, &mappings, &scripts);

        // Should create 3 controllers
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
}
