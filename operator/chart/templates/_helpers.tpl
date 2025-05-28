{{/*
Generate a name for the resource based on the release name and the resource type.
*/}}
{{- define "operator.fullname" -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Get the nuopMode with a default value.
*/}}
{{- define "operator.nuopMode" -}}
{{- default "manager" .Values.deployment.nuopMode -}}
{{- end -}}

{{/*
Generate a simple name for the resource based on the release name.
*/}}
{{- define "operator.name" -}}
{{- printf "%s" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Generate a name for the service account.
*/}}
{{- define "operator.serviceAccountName" -}}
{{- if .Values.rbac.serviceAccount.name }}
{{- printf "%s" .Values.rbac.serviceAccount.name -}}
{{- else }}
{{- include "operator.fullname" . -}}-sa
{{- end -}}
{{- end -}}

{{/*
Generate a name for the cluster role.
*/}}
{{- define "operator.clusterRoleName" -}}
{{- if .Values.rbac.clusterRole.name }}
{{- printf "%s" .Values.rbac.clusterRole.name -}}
{{- else }}
{{- include "operator.fullname" . -}}-clusterrole
{{- end -}}
{{- end -}}

{{/*
Generate a name for the cluster role binding.
*/}}
{{- define "operator.clusterRoleBindingName" -}}
{{- if .Values.rbac.clusterRoleBinding.name }}
{{- printf "%s" .Values.rbac.clusterRoleBinding.name -}}
{{- else }}
{{- include "operator.fullname" . -}}-clusterrolebinding
{{- end -}}
{{- end -}}

{{/*
Generate a namespace for the resources using the release's namespace.
*/}}
{{- define "operator.namespace" -}}
{{- printf "%s" .Release.Namespace -}}
{{- end -}}

{{/*
Generate labels for the resources.
*/}}
{{- define "operator.labels" -}}
app.kubernetes.io/name: {{ include "operator.name" . }}-{{ include "operator.nuopMode" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.deployment.labels }}
{{ toYaml . | nindent 4 }}
{{- end }}
{{- end -}}
