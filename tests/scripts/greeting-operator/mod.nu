#!/usr/bin/env nu

# Greeting Operator - A simple demonstration operator for testing CRD auto-installation
# This operator creates ConfigMaps with greeting messages in different languages

const FINALIZER = "demo.nuop.io/greeting-operator-finalizer"

# Returns operator configuration
def 'main config' [] {
  {
    name: "greeting-operator"
    kind: "GreetingRequest"
    group: "demo.nuop.io"
    version: "v1"
    finalizer: $FINALIZER
    requeue_after_noop: 30
  } | to yaml
}

# Generate greeting in different languages
def generate-greeting [name: string language: string style: string] {
  let greetings = {
    en: {
      formal: $"Good day, ($name). It is a pleasure to meet you."
      informal: $"Hey ($name)! Nice to meet you!"
    }
    es: {
      formal: $"Buenos días, ($name). Es un placer conocerle."
      informal: $"¡Hola ($name)! ¡Qué gusto verte!"
    }
    fr: {
      formal: $"Bonjour, ($name). Enchanté de vous rencontrer."
      informal: $"Salut ($name)! Ravi de te voir!"
    }
    de: {
      formal: $"Guten Tag, ($name). Es ist mir eine Freude, Sie kennenzulernen."
      informal: $"Hallo ($name)! Schön dich zu sehen!"
    }
    ja: {
      formal: $"こんにちは、($name)さん。お会いできて光栄です。"
      informal: $"やあ、($name)! 会えて嬉しいよ!"
    }
  }

  $greetings | get $language | get $style
}

# Get the ConfigMap name for this greeting
def get-configmap-name [greeting_name: string] {
  $"greeting-($greeting_name)"
}

# Create or update the greeting ConfigMap
def create-greeting-configmap [resource: record] {
  let name = $resource.spec.name
  let language = ($resource.spec.language | default "en")
  let style = ($resource.spec.style | default "informal")
  let namespace = $resource.metadata.namespace
  let greeting_request_name = $resource.metadata.name

  let greeting = (generate-greeting $name $language $style)
  let configmap_name = (get-configmap-name $greeting_request_name)

  # Build the ConfigMap
  let configmap = {
    apiVersion: "v1"
    kind: "ConfigMap"
    metadata: {
      name: $configmap_name
      namespace: $namespace
      labels: {
        "app.kubernetes.io/managed-by": "greeting-operator"
        "demo.nuop.io/greeting-request": $greeting_request_name
      }
    }
    data: {
      greeting: $greeting
      name: $name
      language: $language
      style: $style
      timestamp: (date now | format date "%Y-%m-%d %H:%M:%S")
    }
  }

  # Check if ConfigMap already exists
  let existing = (kubectl get configmap $configmap_name -n $namespace -o json | complete)

  if $existing.exit_code == 0 {
    # ConfigMap exists, check if it needs updating
    let existing_cm = ($existing.stdout | from json)
    let existing_greeting = ($existing_cm.data.greeting | default "")

    if $existing_greeting == $greeting {
      print $"✅ ConfigMap '($configmap_name)' is up to date"
      return {changed: false configmap_name: $configmap_name greeting: $greeting}
    } else {
      print $"🔄 Updating ConfigMap '($configmap_name)' with new greeting"
      let apply_result = (($configmap | to yaml) | kubectl apply -f - | complete)
      if $apply_result.exit_code != 0 {
        print $"⚠️  Warning: Failed to update ConfigMap: ($apply_result.stderr)"
      }
      return {changed: true configmap_name: $configmap_name greeting: $greeting}
    }
  } else {
    # ConfigMap doesn't exist, create it
    print $"➕ Creating ConfigMap '($configmap_name)' with greeting"
    let apply_result = (($configmap | to yaml) | kubectl apply -f - | complete)
    if $apply_result.exit_code != 0 {
      print $"⚠️  Warning: Failed to create ConfigMap: ($apply_result.stderr)"
    }
    return {changed: true configmap_name: $configmap_name greeting: $greeting}
  }
}

# Update the status of the GreetingRequest
def update-status [resource: record configmap_name: string greeting: string state: string] {
  let name = $resource.metadata.name
  let namespace = $resource.metadata.namespace

  let status_patch = {
    status: {
      greeting: $greeting
      configMapName: $configmap_name
      lastUpdated: (date now | date to-timezone UTC | format date "%Y-%m-%dT%H:%M:%SZ")
      state: $state
      message: $"Greeting ConfigMap '($configmap_name)' is ($state | str lowercase)"
    }
  }

  # Apply status patch
  let json = ($status_patch | to json -r)
  let patch_result = (
    kubectl patch greetingrequest $name -n $namespace --type=merge --subresource=status --patch $json
    | complete
  )

  if $patch_result.exit_code == 0 {
    print $"✅ Status updated: ($state)"
  } else {
    print $"⚠️  Warning: Failed to update status: ($patch_result.stderr)"
  }
}

# Main reconciliation logic
def 'main reconcile' [] {
  let resource = $in | from yaml

  print $"🎯 Reconciling GreetingRequest: ($resource.metadata.name)"
  print $"   Name: ($resource.spec.name)"
  print $"   Language: ($resource.spec.language | default 'en')"
  print $"   Style: ($resource.spec.style | default 'informal')"

  # Create or update the greeting ConfigMap
  let result = (create-greeting-configmap $resource)

  # Update status
  update-status $resource $result.configmap_name $result.greeting "Ready"

  if $result.changed {
    print "✨ Reconciliation complete - changes made"
    exit 2
  } else {
    print "✅ Reconciliation complete - no changes needed"
    exit 0
  }
}

# Cleanup when GreetingRequest is deleted
def 'main finalize' [] {
  let resource = $in | from yaml

  print $"🗑  Finalizing GreetingRequest: ($resource.metadata.name)"

  let configmap_name = (get-configmap-name $resource.metadata.name)
  let namespace = $resource.metadata.namespace

  # Delete the associated ConfigMap
  let delete_result = (kubectl delete configmap $configmap_name -n $namespace | complete)

  if $delete_result.exit_code == 0 {
    print $"✅ Deleted ConfigMap '($configmap_name)'"
  } else {
    # ConfigMap might not exist, which is fine
    print $"ℹ️  ConfigMap '($configmap_name)' not found or already deleted"
  }

  print "✅ Finalization complete"
  exit 0
}

# Main entry point
def main [] {
  help main
}
