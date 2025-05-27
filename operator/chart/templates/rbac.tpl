{{- if not .Values.existingClusterRole }}
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "operator.serviceAccountName" . }}
  namespace: {{ include "operator.namespace" . }}
{{- end }}

{{- if not .Values.rbac.existingClusterRole }}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: {{ include "operator.clusterRoleName" . }}
rules:
  - apiGroups: [""]
    resources:
      [
        "configmaps",
        "endpoints",
        "namespaces",
        "persistentvolumeclaims",
        "pods",
        "secrets",
        "services",
      ]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["apps"]
    resources: ["deployments", "daemonsets", "replicasets", "statefulsets"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["batch"]
    resources: ["jobs", "cronjobs"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["extensions"]
    resources: ["ingresses"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["kemper.buzz"]
    resources: ["nuoperators"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
{{- end }}

{{- if not .Values.rbac.existingClusterRole }}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: {{ include "operator.clusterRoleBindingName" . }}
subjects:
  - kind: ServiceAccount
    name: {{ include "operator.serviceAccountName" . }}
    namespace: {{ include "operator.namespace" . }}
roleRef:
  kind: ClusterRole
  name: {{ include "operator.clusterRoleName" . }}
  apiGroup: rbac.authorization.k8s.io
{{- end }}
