---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "operator.fullname" . }}-{{ include "operator.nuopMode" . }}
  namespace: {{ include "operator.namespace" . }}
  labels:
    {{- include "operator.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.deployment.replicas | default 1 }}
  selector:
    matchLabels:
      app: {{ include "operator.name" . }}-{{ include "operator.nuopMode" . }}
  template:
    metadata:
      labels:
        app: {{ include "operator.name" . }}-{{ include "operator.nuopMode" . }}
        {{- include "operator.labels" . | nindent 8 }}
    spec:
      serviceAccountName: {{ include "operator.serviceAccountName" . }}
      containers:
        - name: operator-{{ include "operator.nuopMode" . }}
          image: "{{ .Values.deployment.image.repository }}:{{ .Values.deployment.image.tag }}"
          imagePullPolicy: {{ .Values.deployment.image.pullPolicy }}
          env:
            - name: NUOP_MODE
              value: {{ include "operator.nuopMode" . }}
            - name: LOG_FORMAT
              value: {{ .Values.deployment.log.format | quote }}
            - name: LOG_LEVEL
              value: {{ .Values.deployment.log.level | quote }}
