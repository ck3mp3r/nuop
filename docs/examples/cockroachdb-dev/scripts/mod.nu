# Finalizer identifier
const FINALIZER = "github.com/ck3mp3r-nuop-crdb-dev-finalizer"

# Fallback image when spec.template.spec.image is absent
const DEFAULT_IMAGE = "cockroachdb/cockroach:v23.1.11"

# Default service ports
const DEFAULT_GRPC_PORT = 26258
const DEFAULT_HTTP_PORT = 8080
const DEFAULT_SQL_PORT = 26257

# Fixed cache and SQL memory for local development
const CACHE_SIZE = "256MiB"
const MAX_SQL_MEMORY = "256MiB"

# Returns operator config
def 'main config' [] {
  {
    name: "cockroachdb-dev"
    group: "crdb.cockroachlabs.com"
    version: "v1beta1"
    kind: "CrdbCluster"
    labelSelectors: {}
    finalizer: $FINALIZER
    requeue_after_noop: 60
  } | to yaml
}

# Returns the CrdbNode template spec (spec.template.spec), or an empty record
def get-node-spec [spec: record] {
  let template = ($spec | get --optional template | default {})
  $template | get --optional spec | default {}
}

# Reads the service ports from the node spec, applying defaults
def get-ports [spec: record] {
  let node = (get-node-spec $spec)
  {
    grpc: ($node | get --optional grpcPort | default $DEFAULT_GRPC_PORT)
    http: ($node | get --optional httpPort | default $DEFAULT_HTTP_PORT)
    sql: ($node | get --optional sqlPort | default $DEFAULT_SQL_PORT)
  }
}

# Reads the container image from the node spec, applying the default
def get-image [spec: record] {
  let node = (get-node-spec $spec)
  $node | get --optional image | default $DEFAULT_IMAGE
}

# Reads spec.template.spec.dataStore.volumeClaimTemplate.spec, returns null when absent
def get-pvc-spec [spec: record] {
  let node = (get-node-spec $spec)
  let data_store = ($node | get --optional dataStore | default {})
  let claim = ($data_store | get --optional volumeClaimTemplate | default {})
  $claim | get --optional spec
}

# Reads spec.template.spec.startFlags.upsert, returns an empty list when absent
def get-start-flags [spec: record] {
  let node = (get-node-spec $spec)
  let flags = ($node | get --optional startFlags | default {})
  $flags | get --optional upsert | default []
}

# Reads spec.template.spec.resourceRequirements, returns null when absent
def get-resources [spec: record] {
  let node = (get-node-spec $spec)
  $node | get --optional resourceRequirements
}

# Reads the total node count from spec.regions, defaulting to 3 when absent
def get-nodes [spec: record] {
  let regions = ($spec | get --optional regions | default [])
  if ($regions | is-empty) {
    3
  } else {
    $regions | get nodes | math sum
  }
}

# Reads spec.template.metadata.labels, returns an empty record when absent
def get-additional-labels [spec: record] {
  let template = ($spec | get --optional template | default {})
  let metadata = ($template | get --optional metadata | default {})
  $metadata | get --optional labels | default {}
}

# Builds the labels applied to every created resource
def build-labels [name: string, spec: record] {
  {
    "app.kubernetes.io/name": "cockroachdb"
    "app.kubernetes.io/instance": $name
    "app.kubernetes.io/component": "database"
    "app.kubernetes.io/part-of": "cockroachdb"
    "app.kubernetes.io/managed-by": "cockroach-operator"
  } | merge (get-additional-labels $spec)
}

# Builds the pod selector labels
# Matches the full operator: name, instance, component plus spec.template.metadata.labels
def build-selector [labels: record, spec: record] {
  let base = ($labels | select "app.kubernetes.io/name" "app.kubernetes.io/instance" "app.kubernetes.io/component")
  $base | merge (get-additional-labels $spec)
}

# Builds the three service ports shared by both services
def build-service-ports [ports: record] {
  [
    {name: "grpc", port: $ports.grpc, protocol: "TCP"}
    {name: "http", port: $ports.http, protocol: "TCP"}
    {name: "sql", port: $ports.sql, protocol: "TCP"}
  ]
}

# Builds the headless discovery service
# Provides per-pod DNS entries so pods can resolve each other
def build-discovery-service [name: string, namespace: string, ports: record, labels: record, selector: record] {
  {
    apiVersion: "v1"
    kind: "Service"
    metadata: {
      name: $name
      namespace: $namespace
      labels: $labels
      annotations: {
        "prometheus.io/scrape": "true"
        "prometheus.io/path": "_status/vars"
        "prometheus.io/port": ($ports.http | into string)
      }
    }
    spec: {
      clusterIP: "None"
      publishNotReadyAddresses: true
      selector: $selector
      ports: (build-service-ports $ports)
    }
  }
}

# Builds the public ClusterIP service used by clients
# Name matches the full operator: <name>-public
def build-public-service [name: string, namespace: string, ports: record, labels: record, selector: record] {
  {
    apiVersion: "v1"
    kind: "Service"
    metadata: {
      name: $"($name)-public"
      namespace: $namespace
      labels: $labels
    }
    spec: {
      type: "ClusterIP"
      selector: $selector
      ports: (build-service-ports $ports)
    }
  }
}

# Builds the join string that seeds the first three pods
def build-join-string [name: string, namespace: string, nodes: int, grpc_port: int] {
  let count = ([$nodes 3] | math min)
  if $count <= 0 {
    return ""
  }
  0..($count - 1)
  | each {|i| $"($name)-($i).($name).($namespace):($grpc_port)" }
  | str join ","
}

# Builds the cockroach start command
# Matches the full operator: bash -ecx with the advertise-host, ports, and join string
def build-start-command [name: string, namespace: string, ports: record, additional_args: list, join: string] {
  let advertise_host = "--advertise-host=$(POD_NAME)." + $name + "." + $namespace
  let base = [
    "/cockroach/cockroach.sh"
    "start"
    $advertise_host
    "--insecure"
    $"--http-port=($ports.http)"
    $"--sql-addr=:($ports.sql)"
    $"--listen-addr=:($ports.grpc)"
    "--logtostderr=INFO"
    $"--cache=($CACHE_SIZE)"
    $"--max-sql-memory=($MAX_SQL_MEMORY)"
  ]
  let with_args = ($base | append $additional_args)
  let has_join = ($additional_args | any {|arg| $arg | str contains "--join" })
  let full = if $has_join { $with_args } else { $with_args | append $"--join=($join)" }
  ["/bin/bash" "-ecx" $"exec ($full | str join ' ')"]
}

# Builds the StatefulSet that runs the CockroachDB nodes
def build-statefulset [name: string, namespace: string, spec: record, ports: record, image: string, nodes: int, labels: record, selector: record] {
  let additional_args = (get-start-flags $spec)
  let resources = (get-resources $spec)
  let pvc_spec = (get-pvc-spec $spec)

  let join = (build-join-string $name $namespace $nodes $ports.grpc)
  let command = (build-start-command $name $namespace $ports $additional_args $join)

  let env_vars = [
    {name: "COCKROACH_CHANNEL", value: "nuop-cockroachdb-dev"}
    {name: "POD_NAME", valueFrom: {fieldRef: {fieldPath: "metadata.name"}}}
    {name: "GOMAXPROCS", valueFrom: {resourceFieldRef: {resource: "limits.cpu", divisor: 1}}}
    {name: "MEMORY_LIMIT_MIB", valueFrom: {resourceFieldRef: {resource: "limits.memory", divisor: "1Mi"}}}
  ]

  let container = {
    name: "db"
    image: $image
    command: $command
    env: $env_vars
    ports: [
      {name: "grpc", containerPort: $ports.grpc, protocol: "TCP"}
      {name: "http", containerPort: $ports.http, protocol: "TCP"}
      {name: "sql", containerPort: $ports.sql, protocol: "TCP"}
    ]
    readinessProbe: {
      httpGet: {path: "/health?ready=1", port: "http", scheme: "HTTP"}
      initialDelaySeconds: 10
      periodSeconds: 5
      failureThreshold: 2
    }
    volumeMounts: [{name: "datadir", mountPath: "/cockroach/cockroach-data/"}]
  }

  let container = if $resources == null { $container } else { $container | insert resources $resources }

  let pod_spec = if $pvc_spec == null {
    {
      terminationGracePeriodSeconds: 300
      containers: [$container]
      volumes: [{name: "datadir", emptyDir: {}}]
    }
  } else {
    {
      terminationGracePeriodSeconds: 300
      containers: [$container]
    }
  }

  let sts_spec = {
    serviceName: $name
    replicas: $nodes
    podManagementPolicy: "Parallel"
    updateStrategy: {type: "RollingUpdate"}
    selector: {matchLabels: $selector}
    template: {
      metadata: {labels: $labels}
      spec: $pod_spec
    }
  }

  let sts_spec = if $pvc_spec == null {
    $sts_spec
  } else {
    $sts_spec | insert volumeClaimTemplates [{metadata: {name: "datadir"}, spec: $pvc_spec}]
  }

  {
    apiVersion: "apps/v1"
    kind: "StatefulSet"
    metadata: {name: $name, namespace: $namespace, labels: $labels}
    spec: $sts_spec
  }
}

# Returns true when every field in desired exists in actual with the same value
# Extra fields in actual are ignored, so server-side defaults do not cause drift
# Lists of records are matched without regard to order
# Lists of scalars are compared element by element
def contains-subset [desired: any, actual: any] {
  if $desired == null {
    return true
  }
  if $actual == null {
    return false
  }

  let desired_type = ($desired | describe)
  let actual_type = ($actual | describe)

  if ($desired_type | str starts-with "record") {
    if not ($actual_type | str starts-with "record") {
      return false
    }
    let actual_columns = ($actual | columns)
    for key in ($desired | columns) {
      if not ($key in $actual_columns) {
        return false
      }
      if not (contains-subset ($desired | get $key) ($actual | get $key)) {
        return false
      }
    }
    return true
  }

  if ($desired_type | str starts-with "list") or ($desired_type | str starts-with "table") {
    if not (($actual_type | str starts-with "list") or ($actual_type | str starts-with "table")) {
      return false
    }
    let desired_len = ($desired | length)
    if $desired_len != ($actual | length) {
      return false
    }
    if $desired_len == 0 {
      return true
    }

    if (($desired | first | describe) | str starts-with "record") {
      mut remaining = $actual
      for item in $desired {
        let matches = ($remaining | enumerate | where {|entry| contains-subset $item $entry.item })
        if ($matches | length) == 0 {
          return false
        }
        let index = ($matches | first | get index)
        $remaining = ($remaining | drop nth $index)
      }
      return true
    }

    for index in 0..($desired_len - 1) {
      if not (contains-subset ($desired | get $index) ($actual | get $index)) {
        return false
      }
    }
    return true
  }

  # Kubernetes normalizes some scalars to strings (for example Quantity fields
  # such as resourceFieldRef.divisor). Compare scalars by their string form when
  # the types differ so server-side normalization does not register as drift.
  if $desired_type != $actual_type {
    return (($desired | into string) == ($actual | into string))
  }

  $desired == $actual
}

# Returns true when the resource is missing or differs from the desired manifest
def resource-changed [manifest: record, namespace: string] {
  let kind = ($manifest.kind | str lowercase)
  let name = $manifest.metadata.name
  let existing = (kubectl get $kind $name -n $namespace -o json | complete)
  if $existing.exit_code != 0 {
    return true
  }
  not (contains-subset $manifest ($existing.stdout | from json))
}

# Applies the manifest when it is missing or differs, returns true when it changed
def apply-resource [manifest: record, namespace: string] {
  let kind = $manifest.kind
  let name = $manifest.metadata.name

  if not (resource-changed $manifest $namespace) {
    print $"✅ ($kind)/($name) is up to date"
    return false
  }

  let result = (($manifest | to yaml) | kubectl apply --server-side -f - | complete)
  if $result.exit_code != 0 {
    print $"❌ Failed to apply ($kind)/($name): ($result.stderr)"
    exit 1
  }

  print $"➕ Applied ($kind)/($name)"
  true
}

# Reads metadata.generation from the watch event, falling back to a live get
def get-generation [parsed: record] {
  let generation = ($parsed.metadata | get --optional generation)
  if $generation != null {
    return $generation
  }

  let name = $parsed.metadata.name
  let namespace = $parsed.metadata.namespace
  let result = (kubectl get crdbcluster $name -n $namespace -o jsonpath='{.metadata.generation}' | complete)
  if $result.exit_code != 0 {
    print $"❌ Failed to read generation: ($result.stderr)"
    exit 1
  }

  $result.stdout | str trim | into int
}

# Patches the CrdbCluster status subresource
def patch-status [name: string, namespace: string, generation: int, patch: record] {
  let payload = ($patch | merge {observedGeneration: $generation})
  let json = ({status: $payload} | to json -r)
  let result = (
    kubectl patch crdbcluster $name -n $namespace --type=merge --subresource=status --patch $json
    | complete
  )

  if $result.exit_code != 0 {
    print $"❌ Failed to patch status: ($result.stderr)"
    exit 1
  }

  print $"🔄 Patched status: ($payload | to json -r)"
}

# Reads the ClusterReady condition status from the CrdbCluster status
# Returns "True", "False", or "" when the condition is absent
def get-cluster-ready [status: record] {
  let conditions = ($status | get --optional conditions | default [])
  let ready = ($conditions | where {|condition| ($condition | get --optional type) == "ClusterReady" })
  if ($ready | is-empty) {
    ""
  } else {
    $ready | first | get --optional status | default ""
  }
}

# Patches the ClusterReady condition on the CrdbCluster status
def patch-cluster-ready [name: string, namespace: string, generation: int, ready: bool] {
  let value = if $ready { "True" } else { "False" }
  patch-status $name $namespace $generation {
    conditions: [
      {
        type: "ClusterReady"
        status: $value
        lastTransitionTime: (date now | date to-timezone UTC | format date "%Y-%m-%dT%H:%M:%SZ")
      }
    ]
  }
}

# Returns true when the StatefulSet reports all requested replicas as ready
def statefulset-ready [name: string, namespace: string, replicas: int] {
  let result = (kubectl get statefulset $name -n $namespace -o json | complete)
  if $result.exit_code != 0 {
    return false
  }
  let statefulset = ($result.stdout | from json)
  let ready = ($statefulset.status | get --optional readyReplicas | default 0)
  $ready == $replicas
}

# Reconciliation logic for create/update events
def handle-reconcile [parsed: record] {
  let name = $parsed.metadata.name
  let namespace = $parsed.metadata.namespace
  let spec = ($parsed | get --optional spec | default {})
  let status = ($parsed | get --optional status | default {})
  let generation = (get-generation $parsed)

  let ports = (get-ports $spec)
  let image = (get-image $spec)
  let nodes = (get-nodes $spec)
  let labels = (build-labels $name $spec)
  let selector = (build-selector $labels $spec)

  let discovery_service = (build-discovery-service $name $namespace $ports $labels $selector)
  let public_service = (build-public-service $name $namespace $ports $labels $selector)
  let statefulset = (build-statefulset $name $namespace $spec $ports $image $nodes $labels $selector)

  let current_ready = (get-cluster-ready $status)
  let ready = (statefulset-ready $name $namespace $nodes)

  mut changed = false

  if (not $ready) and $current_ready != "False" {
    patch-cluster-ready $name $namespace $generation false
    $changed = true
  }

  if (apply-resource $discovery_service $namespace) {
    $changed = true
  }
  if (apply-resource $public_service $namespace) {
    $changed = true
  }
  if (apply-resource $statefulset $namespace) {
    $changed = true
  }

  let current_image = ($status | get --optional image | default "")
  if $current_image != $image {
    patch-status $name $namespace $generation {image: $image}
    $changed = true
  }

  if $ready and $current_ready != "True" {
    patch-cluster-ready $name $namespace $generation true
    $changed = true
  }

  if $changed {
    exit 2
  } else {
    exit 0
  }
}

# Handles deletion of the CrdbCluster by removing the managed resources
# PVCs are left in place so data survives a cluster deletion
def handle-finalize [parsed: record] {
  let name = $parsed.metadata.name
  let namespace = $parsed.metadata.namespace

  print $"🗑 Finalizing CrdbCluster ($name) in ($namespace)"

  let targets = [
    {kind: "statefulset", name: $name}
    {kind: "service", name: $name}
    {kind: "service", name: $"($name)-public"}
  ]

  for target in $targets {
    let result = (kubectl delete $target.kind $target.name -n $namespace --ignore-not-found | complete)
    if $result.exit_code != 0 {
      print $"❌ Failed to delete ($target.kind)/($target.name): ($result.stderr)"
      exit 1
    }
    print $"❌ Deleted ($target.kind)/($target.name)"
  }

  exit 0
}

# Main dispatch logic
def 'main reconcile' [] {
  let parsed = $in | from yaml
  handle-reconcile $parsed
}

def 'main finalize' [] {
  let parsed = $in | from yaml
  handle-finalize $parsed
}

def main [] {
  help main
}
